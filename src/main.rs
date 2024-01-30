fn main() {
    use std::io::Cursor;
    use tantivy::{schema::Schema, Document, Index, IndexSettings, IndexSortByField, Order};
    use tantivy_common::BinarySerializable;

    fn test_writer_commit() {
        let serialized_schema = r#"
      [{"name":"category","type":"text","options":{"indexing":{"record":"position","fieldnorms":true,"tokenizer":"default"},"stored":true,"fast":false}},{"name":"description","type":"text","options":{"indexing":{"record":"position","fieldnorms":true,"tokenizer":"default"},"stored":true,"fast":false}},{"name":"rating","type":"i64","options":{"indexed":true,"fieldnorms":false,"fast":true,"stored":true}},{"name":"in_stock","type":"bool","options":{"indexed":true,"fieldnorms":false,"fast":true,"stored":true}},{"name":"metadata","type":"json_object","options":{"stored":true,"indexing":{"record":"position","fieldnorms":true,"tokenizer":"default"},"fast":false,"expand_dots_enabled":true}},{"name":"id","type":"i64","options":{"indexed":true,"fieldnorms":true,"fast":true,"stored":true}},{"name":"ctid","type":"u64","options":{"indexed":true,"fieldnorms":true,"fast":true,"stored":true}}]
    "#;

        let schema: Schema = serde_json::from_str(&serialized_schema).unwrap();
        let settings = IndexSettings {
            sort_by_field: Some(IndexSortByField {
                field: "id".into(),
                order: Order::Asc,
            }),
            ..Default::default()
        };

        let temp_dir = tempfile::Builder::new().tempdir().unwrap();

        let index = Index::builder()
            .schema(schema)
            .settings(settings)
            .create_in_dir(&temp_dir.path())
            .unwrap();

        let mut writer = index.writer(500_000_000).unwrap();

        // This is a string representation of the document bytes that I am sending through IPC.
        let document_bytes: Vec<u8> = serde_json::from_str("[135,5,0,0,0,2,1,0,0,0,0,0,0,0,1,0,0,0,0,152,69,114,103,111,110,111,109,105,99,32,109,101,116,97,108,32,107,101,121,98,111,97,114,100,2,0,0,0,2,4,0,0,0,0,0,0,0,0,0,0,0,0,139,69,108,101,99,116,114,111,110,105,99,115,3,0,0,0,9,1,4,0,0,0,8,123,34,99,111,108,111,114,34,58,34,83,105,108,118,101,114,34,44,34,108,111,99,97,116,105,111,110,34,58,34,85,110,105,116,101,100,32,83,116,97,116,101,115,34,125,5,0,0,0,1,1,0,0,0,0,0,0,0]").unwrap();

        let document_from_bytes: Document =
            BinarySerializable::deserialize(&mut Cursor::new(document_bytes)).unwrap();

        // This is a json representation of the above that I'm including here for readability.
        // This was generated with `println!(serde_json::to_string(document_from_bytes).unwrap())`.
        let document_json = r#"
            {"field_values":[{"field":5,"value":1},{"field":1,"value":"Ergonomic metal keyboard"},{"field":2,"value":4},{"field":0,"value":"Electronics"},{"field":3,"value":true},{"field":4,"value":{"color":"Silver","location":"United States"}},{"field":5,"value":1}]}
        "#;

        // To prove that the document_json and the document_from_bytes represent the same Document,
        // we assert their equality here. This is expected to pass.
        assert_eq!(
            document_json.trim(),
            serde_json::to_string(&document_from_bytes).unwrap().trim()
        );

        writer.add_document(document_from_bytes).unwrap();

        // We expect an error here on commit: ErrorInThread("Any { .. }")
        writer.commit().unwrap();
    }

    test_writer_commit();
}
