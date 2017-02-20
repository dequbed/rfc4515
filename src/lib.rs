#[macro_use]
extern crate nom;
extern crate asnom;

use std::default::Default;

use asnom::common::TagClass;
use asnom::structures::{Tag, Sequence, OctetString, ExplicitTag};

named!(filter <Tag>, delimited!(
    char!('('),
    flat_map!(take_until_s!(")"), filtercomp),
    char!(')')));

named!(filterlist <Vec<Tag> >, many1!(filter));

named!(filtercomp <Tag>, alt!(and_f | or_f | not_f | item));

named!(and_f <Tag>,
    do_parse!(
        char!('&') >>
        out: map!(filterlist,
            | tagv: Vec<Tag> | -> Tag {
                Tag::Sequence(Sequence {
                    id: 0,
                    class: TagClass::Context,
                    inner: tagv,
                })
            }
        ) >>
        (out)
    )
);

named!(or_f <Tag>,
    do_parse!(
        char!('|') >>
        out: map!(filterlist,
            | tagv: Vec<Tag> | -> Tag {
                Tag::Sequence(Sequence {
                    id: 1,
                    class: TagClass::Context,
                    inner: tagv,
                })
            }
        ) >>
        (out)
    )
);

named!(not_f <Tag>,
    do_parse!(
        char!('!') >>
        out: map!(filter,
            | tag: Tag | -> Tag {
                Tag::ExplicitTag(ExplicitTag {
                    class: TagClass::Context,
                    id: 2,
                    inner: Box::new(tag),
                })
            }
        ) >>
        (out)
    )
);

named!(item <Tag>, alt!(present | simple));

named!(simple <Tag>, do_parse!(
    k: take_until_either!("<=~>") >>
    id: s_match_type >>
    v: call!(nom::rest) >>
    (Tag::Sequence(Sequence {
        class: TagClass::Context,
        id: id,
        inner: vec![
            Tag::OctetString(OctetString {
                inner: k.to_vec(),
                .. Default::default()
            }),
            Tag::OctetString(OctetString {
                inner: v.to_vec(),
                .. Default::default()
            })
        ],
    }))
));

named!(present <Tag>, do_parse!(
    k: take_until_s!("=*") >>
    (Tag::OctetString(OctetString {
        class: TagClass::Context,
        id: 7,
        inner: k.to_vec()
    }))
));

named!(s_match_type <u64>, alt!(equal | approx | geq | leq));

named!(equal <u64>, do_parse!(
        tag!("=") >>
        (3) // ID of an EqualityMatch
));

named!(approx <u64>, do_parse!(
        tag!("~=") >>
        (8) // ID of an EqualityMatch
));

named!(geq <u64>, do_parse!(
        tag!(">=") >>
        (5) // ID of an EqualityMatch
));

named!(leq <u64>, do_parse!(
        tag!("<=") >>
        (6) // ID of an EqualityMatch
));


#[cfg(test)]
mod tests {

    use super::*;

    use std::default::Default;
    use nom::IResult;
    use asnom::common::TagClass;
    use asnom::structures::{Tag, OctetString, Sequence, ExplicitTag};

    #[test]
    fn present() {
        let f = &b"(objectClass=*)"[..];

        let tag = Tag::OctetString(OctetString {
            class: TagClass::Context,
            id: 7,
            inner: vec![
                0x6f, 0x62, 0x6a, 0x65, 0x63, 0x74, 0x43, 0x6c, 0x61, 0x73, 0x73
            ],
        });

        let o: IResult<&[u8], Tag> = super::filter(f);
        let left = Vec::new();
        assert_eq!(o, IResult::Done(&left[..], tag));
    }

    #[test]
    fn simple() {
        let f = &b"(cn=Babs Jensen)"[..];

        let tag = Tag::Sequence(Sequence {
            class: TagClass::Context,
            id: 3,
            inner: vec![
                   Tag::OctetString(OctetString {
                       inner: vec![0x63, 0x6e],
                       .. Default::default()
                   }),
                   Tag::OctetString(OctetString {
                       inner: vec![0x42, 0x61, 0x62, 0x73, 0x20, 0x4a, 0x65, 0x6e, 0x73, 0x65, 0x6e],
                        .. Default::default()
                   })
            ]
        });

        let o: IResult<&[u8], Tag> = super::filter(f);
        let left = Vec::new();
        assert_eq!(o, IResult::Done(&left[..], tag));
    }

    #[test]
    fn not() {
        let f = &b"(!(cn=Tim Howes))"[..];

        let tag = Tag::ExplicitTag(ExplicitTag {
            class: TagClass::Context,
            id: 2,
            inner: Box::new(Tag::Sequence(Sequence {
                inner: vec![
                    Tag::OctetString(OctetString {
                        inner: vec![0x63, 0x6e],
                        .. Default::default()
                    }),
                    Tag::OctetString(OctetString {
                        inner: vec![0x54, 0x69, 0x6d, 0x20, 0x48, 0x6f, 0x77, 0x65, 0x73],
                        .. Default::default()
                    })
                ],
                .. Default::default()
            })),
        });

        let o: IResult<&[u8], Tag> = super::filter(f);
        let left = Vec::new();
        assert_eq!(o, IResult::Done(&left[..], tag));
    }
}
