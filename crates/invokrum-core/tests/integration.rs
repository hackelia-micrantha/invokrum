use invokrum_core::SCHEMA_FAMILY;

#[test]
fn public_schema_contract_is_available_to_consumers() {
    assert_eq!(SCHEMA_FAMILY, "invokrum.dev/v1");
}
