uniffi::setup_scaffolding!();
//cargo build --lib
//cargo run --bin uniffi-bindgen generate --library ./target/debug/libphilologus_fulltext.dylib --language swift --out-dir ./bindings
use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, ReloadPolicy};

#[derive(uniffi::Object)]
pub struct FullTextWrapper {
    index: Index,
}

#[uniffi::export]
impl FullTextWrapper {
    #[uniffi::constructor]
    pub fn new(tantivy_index_path: &str) -> Self {
        let index = Index::open_in_dir(tantivy_index_path).unwrap();
        Self { index }
    }

    pub fn full_text_query(&self, query: &str, page: u32, limit: u32) -> Vec<u32> {
        //let db = req.app_data::<SqlitePool>().unwrap();
        //let index = req.app_data::<tantivy::Index>().unwrap();

        //let limit = 20;
        let offset = page * 20;

        // let word_id_field = index.schema().get_field("word_id").unwrap();
        // let lemma_field = index.schema().get_field("lemma").unwrap();
        let lexicon_field = self.index.schema().get_field("lexicon").unwrap();
        let definition_field = self.index.schema().get_field("definition").unwrap();

        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .unwrap();

        let searcher = reader.searcher();
        let mut query_parser = QueryParser::for_index(
            &self.index,
            //this vector contains default fields used if field is not specified in query
            vec![lexicon_field, definition_field],
        );
        query_parser.set_conjunction_by_default(); // AND by default

        let mut res = vec![];
        // let mut res = FullTextResponse {
        //     ftresults: vec![],
        //     error: String::from(""),
        //     count: 0,
        //     page: page,
        //     limit,
        // };

        // full-text index should be all lowercase, but use uppercase for AND and OR
        let ft_query = query
            .to_lowercase()
            .replace(" and ", " AND ")
            .replace(" or ", " OR ");
        // ft_query = hgk_strip_diacritics(
        //     ft_query
        //         .replace(" and ", " AND ")
        //         .replace(" or ", " OR ")
        //         .as_str(),
        //     0xFFFFFFFF,
        // );

        let my_collector = (
            Count,
            TopDocs::with_limit(limit as usize).and_offset(offset as usize),
        );
        match query_parser.parse_query(&ft_query) {
            //"carry AND (lexicon:slater OR lexicon:lewisshort)") {
            Ok(query) => match searcher.search(&query, &my_collector) {
                Ok((_count, top_docs)) => {
                    for (_score, doc_address) in top_docs {
                        match searcher.doc::<TantivyDocument>(doc_address) {
                            Ok(retrieved_doc) => {
                                let mut word_id_value: u32 = 0;
                                let mut lexicon_value: String = String::from("");

                                for (field, field_values) in retrieved_doc.get_sorted_field_values()
                                {
                                    match self.index.schema().get_field_name(field) {
                                        "lexicon" => {
                                            lexicon_value =
                                                field_values[0].as_str().unwrap_or("").to_string()
                                        }
                                        "word_id" => {
                                            word_id_value = field_values[0]
                                                .as_u64()
                                                .unwrap_or(0)
                                                .try_into()
                                                .unwrap_or(0)
                                        }
                                        _ => continue,
                                    }
                                }
                                // skip entry if these values aren't found
                                // this shouldn't happen
                                if word_id_value == 0 || lexicon_value.is_empty() {
                                    continue;
                                }

                                // let d = get_def_by_seq(db, word_id_value)
                                //     .await
                                //     .map_err(map_sqlx_error)?;

                                // let entry = LexEntry {
                                //     id: d.seq as u64,
                                //     lemma: d.word,
                                //     lex: lexicon_value,
                                //     def: d.def,
                                // };

                                //res.ftresults.push(entry);
                                res.push(word_id_value);
                            }
                            Err(e) => {
                                println!("Full-text error retrieving document: {:?}", e);
                                //res.error = format!("Full-text error retrieving document: {:?}", e);
                            }
                        }
                    }
                    //res.count = count;
                }
                Err(e) => {
                    println!("Full-text error searching document: {:?}", e);
                    //res.error = format!("Full-text error searching document: {:?}", e);
                }
            },
            Err(e) => {
                println!("Error parsing full-text query: {:?}", e);
                //res.error = format!("Error parsing full-text query: {:?}", e);
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ft_test() {
        let index_path = "../philologus-actix-web/tantivy-data";
        let query = "example";
        let page = 1;

        let ft = FullTextWrapper::new(index_path);
        let res = ft.full_text_query(query, page, 20);

        assert_eq!(res.len(), 20);
    }
}
