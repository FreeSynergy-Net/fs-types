//! Registry values (RFC 9553 `kind`, RFC 8984 `relativeTo`) are kept: an unknown word comes back unchanged, a known
//! word is always its variant, and `ContactAddress` takes every absolute URI except the unsafe schemes.

use capnp::message::Builder;
use fs_types::types_capnp::{address_component, name_component};
use fs_types::{
    Address, AddressKind, ContactAddress, Link, Location, LocationParts, Name, NameKind,
    RelativeTo, TypeProblem,
};

const VENDOR: &str = "example.com:baz";

const NAME_WORDS: [&str; 8] = [
    "title",
    "given",
    "given2",
    "surname",
    "surname2",
    "credential",
    "generation",
    "separator",
];
const ADDRESS_WORDS: [&str; 17] = [
    "room",
    "apartment",
    "floor",
    "building",
    "number",
    "name",
    "block",
    "subdistrict",
    "district",
    "locality",
    "region",
    "postcode",
    "country",
    "direction",
    "landmark",
    "postOfficeBox",
    "separator",
];

#[test]
fn a_vendor_name_kind_goes_in_and_comes_out_the_same() {
    let name = Name::new("", &[(NameKind::from_text(VENDOR), "Ada")]).unwrap();
    assert_eq!(name.components()[0].0, NameKind::Other(VENDOR.to_owned()));
    assert_eq!(name.components()[0].0.as_str(), VENDOR);
    assert_eq!(name.components()[0].0.to_string(), VENDOR);
}

#[test]
fn a_vendor_address_kind_goes_in_and_comes_out_the_same() {
    let a = Address::new(&[(AddressKind::from_text(VENDOR), "X")], "", "").unwrap();
    assert_eq!(a.components()[0].0, AddressKind::Other(VENDOR.to_owned()));
    assert_eq!(a.components()[0].0.as_str(), VENDOR);
}

#[test]
fn a_vendor_relative_to_stays_on_the_location() {
    // Regression: `LocationParts` -> `Location` used to refuse the whole location (`NotInList`).
    let parts = LocationParts::new()
        .with_name("Gate")
        .with_relative_to(VENDOR);
    let l = Location::from_parts(&parts).unwrap();
    assert_eq!(l.relative_to(), Some(&RelativeTo::Other(VENDOR.to_owned())));
    assert_eq!(l.to_parts(), parts);
    assert_eq!(l.to_parts().relative_to(), Some(VENDOR));
}

#[test]
fn every_known_word_is_its_variant_and_writes_back_the_same() {
    for w in NAME_WORDS {
        let k = NameKind::from_text(w);
        assert!(!matches!(k, NameKind::Other(_)), "{w}");
        assert_eq!(k.as_str(), w);
    }
    for w in ADDRESS_WORDS {
        let k = AddressKind::from_text(w);
        assert!(!matches!(k, AddressKind::Other(_)), "{w}");
        assert_eq!(k.as_str(), w);
    }
    assert_eq!(RelativeTo::from_text("start"), RelativeTo::Start);
    assert_eq!(RelativeTo::from_text("end"), RelativeTo::End);
    assert_eq!(NameKind::from_text("given2"), NameKind::Given2);
    assert_eq!(
        AddressKind::from_text("postOfficeBox"),
        AddressKind::PostOfficeBox
    );
}

#[test]
fn other_never_holds_a_known_word() {
    // The words are case-sensitive: a different case is an unknown word and stays as it is.
    assert_eq!(
        NameKind::from_text("Given"),
        NameKind::Other("Given".to_owned())
    );
    assert_eq!(
        RelativeTo::from_text("START"),
        RelativeTo::Other("START".to_owned())
    );
    assert_eq!(
        AddressKind::from_text("postofficebox"),
        AddressKind::Other("postofficebox".to_owned())
    );
    // A hand-made `Other` of a known word becomes the variant in every value object.
    let name = Name::new("", &[(NameKind::Other("given".to_owned()), "Ada")]).unwrap();
    assert_eq!(name.components()[0].0, NameKind::Given);
    let a = Address::new(&[(AddressKind::Other("locality".to_owned()), "X")], "", "").unwrap();
    assert_eq!(a.components()[0].0, AddressKind::Locality);
    let l = Location::new()
        .with_relative_to(Some(RelativeTo::Other("end".to_owned())))
        .unwrap();
    assert_eq!(l.relative_to(), Some(&RelativeTo::End));
    assert_eq!(
        NameKind::Other("given".to_owned()).checked().unwrap(),
        NameKind::Given
    );
}

#[test]
fn a_bad_word_is_refused_only_at_the_limits_of_abuse() {
    for (word, problem) in [
        ("", TypeProblem::Empty),
        ("a\u{0}b", TypeProblem::Character),
        (&"x".repeat(1025), TypeProblem::TooLong),
    ] {
        assert_eq!(
            Name::new("", &[(NameKind::Other(word.to_owned()), "v")]).err(),
            Some(problem)
        );
        assert_eq!(
            Address::new(&[(AddressKind::Other(word.to_owned()), "v")], "", "").err(),
            Some(problem)
        );
        assert_eq!(
            Location::new()
                .with_relative_to(Some(RelativeTo::Other(word.to_owned())))
                .err(),
            Some(problem)
        );
    }
    // Reading itself never refuses.
    assert_eq!(
        NameKind::from_text("a b"),
        NameKind::Other("a b".to_owned())
    );
}

#[test]
fn the_wire_carries_the_kind_as_text() {
    let mut msg = Builder::new_default();
    let mut c = msg.init_root::<name_component::Builder<'_>>();
    c.set_kind(NameKind::from_text(VENDOR).as_str());
    c.set_value("Ada");
    let r = c.into_reader();
    let kind = r.get_kind().unwrap().to_str().unwrap();
    assert_eq!(
        NameKind::from_text(kind),
        NameKind::Other(VENDOR.to_owned())
    );

    let mut msg = Builder::new_default();
    let mut c = msg.init_root::<address_component::Builder<'_>>();
    c.set_kind(AddressKind::Postcode.as_str());
    let r = c.into_reader();
    let kind = r.get_kind().unwrap().to_str().unwrap();
    assert_eq!(kind, "postcode");
    assert_eq!(AddressKind::from_text(kind), AddressKind::Postcode);
}

#[test]
fn contact_address_takes_the_rfc_9553_examples() {
    for ok in [
        "webcal://calendar.example.com/calA.ics",
        "ldap://ldap.example/o=Example%20Tech,ou=Engineering",
        "tel:+1-555-555-5555;ext=5555",
        "irc://irc.example.org/channel",
    ] {
        let a = ContactAddress::parse(ok).unwrap();
        assert_eq!(a.canonical(), ok, "kept as written");
    }
}

#[test]
fn contact_address_and_link_share_one_list_of_unsafe_schemes() {
    for bad in [
        "javascript:alert(1)",
        "JavaScript:alert(1)",
        "vbscript:x",
        "data:text/html;base64,AAAA",
    ] {
        assert_eq!(
            ContactAddress::parse(bad).err(),
            Some(TypeProblem::UnsafeScheme),
            "{bad}"
        );
        assert_eq!(
            <Link as fs_types::BasicValue>::parse(bad).err(),
            Some(TypeProblem::UnsafeScheme),
            "{bad}"
        );
    }
}

#[test]
fn a_vendor_word_with_a_long_domain_is_kept() {
    // RFC 9553 section 1.8.1 limits neither the prefix nor the name; a domain has up to 253 characters, and the
    // limit of a word is the one of the other values (1024), not 128.
    let word = format!("{}.example.com:baz", "a".repeat(250));
    assert!(Name::new("", &[(NameKind::Other(word.clone()), "v")]).is_ok());
    assert_eq!(
        NameKind::Other(word.clone()).checked().unwrap().as_str(),
        word
    );
    let edge = "x".repeat(1024);
    assert!(AddressKind::Other(edge).checked().is_ok());
    assert_eq!(
        RelativeTo::Other("x".repeat(1025)).checked(),
        Err(TypeProblem::TooLong)
    );
}

#[test]
fn an_empty_relative_to_reads_as_not_set() {
    // RFC 8984 section 4.2.5: an unusable value counts as omitted; it must not refuse the whole location.
    let parts = LocationParts::new().with_name("Gate").with_relative_to("");
    let l = Location::from_parts(&parts).unwrap();
    assert_eq!(l.name(), Some("Gate"));
    assert_eq!(l.relative_to(), None);
    // the building side still refuses an empty word
    assert_eq!(
        Location::new()
            .with_relative_to(Some(RelativeTo::Other(String::new())))
            .err(),
        Some(TypeProblem::Empty)
    );
}

#[test]
fn the_vendor_word_is_not_lost_by_a_conversion() {
    // `AsRef<str>` is the one public string view; `Other` gives its word, not the variant name "other".
    assert_eq!(NameKind::Other(VENDOR.to_owned()).as_ref(), VENDOR);
    assert_eq!(AddressKind::Other(VENDOR.to_owned()).as_ref(), VENDOR);
    assert_eq!(RelativeTo::Other(VENDOR.to_owned()).as_ref(), VENDOR);
    assert_eq!(NameKind::Given2.as_ref(), "given2");
    assert_eq!(AddressKind::PostOfficeBox.as_ref(), "postOfficeBox");
    assert_eq!(RelativeTo::Start.as_ref(), "start");
}
