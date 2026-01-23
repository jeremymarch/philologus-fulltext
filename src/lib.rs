uniffi::setup_scaffolding!();
//cargo build --lib
//cargo run --bin uniffi-bindgen generate --library ./target/debug/libphilologus_fulltext.dylib --language swift --out-dir ./bindings
use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, ReloadPolicy};

pub fn get_index(tantivy_index_path: &str) -> Index {
    // fix tantivy
    // let tantivy_index_path = std::env::var("TANTIVY_INDEX_PATH").unwrap_or_else(|_| {
    //     panic!("Environment variable for tantivy index path not set: TANTIVY_INDEX_PATH.")
    // });

    Index::open_in_dir(tantivy_index_path).unwrap()
}

pub fn full_text_query(query: &str, page: usize, index: &Index) -> Vec<u32> {
    //let db = req.app_data::<SqlitePool>().unwrap();
    //let index = req.app_data::<tantivy::Index>().unwrap();

    let limit = 20;
    let offset = page * 20;

    // let word_id_field = index.schema().get_field("word_id").unwrap();
    // let lemma_field = index.schema().get_field("lemma").unwrap();
    let lexicon_field = index.schema().get_field("lexicon").unwrap();
    let definition_field = index.schema().get_field("definition").unwrap();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()
        .unwrap();

    let searcher = reader.searcher();
    let mut query_parser = QueryParser::for_index(
        index,
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

    let my_collector = (Count, TopDocs::with_limit(limit).and_offset(offset));
    match query_parser.parse_query(&ft_query) {
        //"carry AND (lexicon:slater OR lexicon:lewisshort)") {
        Ok(query) => match searcher.search(&query, &my_collector) {
            Ok((_count, top_docs)) => {
                for (_score, doc_address) in top_docs {
                    match searcher.doc::<TantivyDocument>(doc_address) {
                        Ok(retrieved_doc) => {
                            let mut word_id_value: u32 = 0;
                            let mut lexicon_value: String = String::from("");

                            for (field, field_values) in retrieved_doc.get_sorted_field_values() {
                                match index.schema().get_field_name(field) {
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

#[uniffi::export]
pub fn real_full_text_query(query: &str, page: u32, index_path: &str) -> Vec<u32> {
    let index = get_index(index_path);
    full_text_query(query, page as usize, &index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ft_test() {
        let index = get_index("../philologus-actix-web/tantivy-data");
        let res = full_text_query("example", 1, &index);

        assert_eq!(res.len(), 20);
    }

    #[test]
    fn ft_test2() {
        let index_path = "../philologus-actix-web/tantivy-data";
        let query = "example";
        let page = 1;

        let res = real_full_text_query(query, page, index_path);

        assert_eq!(res.len(), 20);
    }
}
