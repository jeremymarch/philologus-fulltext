uniffi::setup_scaffolding!();
//cargo build --lib
//cargo run --bin uniffi-bindgen generate --library ./target/debug/libphilologus_fulltext.dylib --language swift --out-dir ./bindings
use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::snippet::SnippetGenerator;
use tantivy::tokenizer::*;
use tantivy::{Index, ReloadPolicy};

#[derive(uniffi::Record)]
pub struct SnippetResult {
    pub word_id: u32,
    pub lemma: String,
    pub snippet: String,
    pub lexicon: String,
}

#[derive(uniffi::Object)]
pub struct FullTextWrapper {
    index: Index,
}

#[uniffi::export]
impl FullTextWrapper {
    #[uniffi::constructor]
    pub fn new(tantivy_index_path: &str) -> Self {
        let index = Index::open_in_dir(tantivy_index_path).unwrap();
        let en_stem_analyzer = TextAnalyzer::builder(SimpleTokenizer::default())
            //.filter(StopWordFilter::new(Language::Greek))
            .filter(LowerCaser)
            .filter(NoDiacritcs)
            .filter(Stemmer::new(Language::English))
            //.filter(Stemmer::new(Language::Greek))
            .build();

        index.tokenizers().register("el_stem", en_stem_analyzer);
        Self { index }
    }

    // pub fn snippet_query(&self, query_str: &str, page: u32, limit: u32) -> Vec<String> {
    //     let query = query_parser.parse_query(query_str).unwrap();
    //     let mut snippet_generator = SnippetGenerator::create(&searcher, &*query, text_field)?;
    //     snippet_generator.set_max_num_chars(100);
    //     let snippet = snippet_generator.snippet_from_doc(&doc);
    //     let snippet_html: String = snippet.to_html();

    //     snippet_html
    // }

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

    pub fn full_text_snippets(&self, query: &str, page: u32, limit: u32) -> Vec<SnippetResult> {
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
            Ok(query) => {
                let mut snippet_generator =
                    SnippetGenerator::create(&searcher, &query, definition_field).unwrap();
                snippet_generator.set_max_num_chars(100);
                match searcher.search(&query, &my_collector) {
                    Ok((_count, top_docs)) => {
                        for (_score, doc_address) in top_docs {
                            match searcher.doc::<TantivyDocument>(doc_address) {
                                Ok(retrieved_doc) => {
                                    let mut word_id_value: u32 = 0;
                                    let mut lexicon_value: String = String::from("");
                                    let mut lemma_value: String = String::from("");

                                    for (field, field_values) in
                                        retrieved_doc.get_sorted_field_values()
                                    {
                                        match self.index.schema().get_field_name(field) {
                                            "word_id" => {
                                                word_id_value = field_values[0]
                                                    .as_u64()
                                                    .unwrap_or(0)
                                                    .try_into()
                                                    .unwrap_or(0)
                                            }
                                            "lemma" => {
                                                lemma_value = field_values[0]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string()
                                            }
                                            "lexicon" => {
                                                lexicon_value = field_values[0]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string()
                                            }
                                            _ => continue,
                                        }
                                    }
                                    // skip entry if these values aren't found
                                    // this shouldn't happen
                                    if word_id_value == 0
                                        || lexicon_value.is_empty()
                                        || lemma_value.is_empty()
                                    {
                                        continue;
                                    }

                                    let snippet =
                                        snippet_generator.snippet_from_doc(&retrieved_doc);
                                    let html_snippet = snippet.to_html();
                                    res.push(SnippetResult {
                                        word_id: word_id_value,
                                        lemma: lemma_value,
                                        snippet: html_snippet,
                                        lexicon: lexicon_value,
                                    });
                                    //println!("Snippet: {}", html_snippet);
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
                                    //res.push(word_id_value);
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
                }
            }
            Err(e) => {
                println!("Error parsing full-text query: {:?}", e);
                //res.error = format!("Error parsing full-text query: {:?}", e);
            }
        }
        res
    }
}

use polytonic_greek::hgk_strip_diacritics;
use std::mem;

use tantivy::tokenizer::{Token, TokenFilter, TokenStream, Tokenizer};

/// Token filter that removes diacritics from terms.
#[derive(Clone)]
pub struct NoDiacritcs;

impl TokenFilter for NoDiacritcs {
    type Tokenizer<T: Tokenizer> = DiacriticFilter<T>;

    fn transform<T: Tokenizer>(self, tokenizer: T) -> Self::Tokenizer<T> {
        DiacriticFilter {
            tokenizer,
            buffer: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct DiacriticFilter<T> {
    tokenizer: T,
    buffer: String,
}

impl<T: Tokenizer> Tokenizer for DiacriticFilter<T> {
    type TokenStream<'a> = DiacriticTokenStream<'a, T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        self.buffer.clear();
        DiacriticTokenStream {
            tail: self.tokenizer.token_stream(text),
            buffer: &mut self.buffer,
        }
    }
}

pub struct DiacriticTokenStream<'a, T> {
    buffer: &'a mut String,
    tail: T,
}

// writes a lowercased version of text into output.
fn to_diacritic_free_unicode(text: &str, output: &mut String) {
    output.clear();
    output.reserve(50);
    // for c in text.chars() {
    //     // Contrary to the std, we do not take care of sigma special case.
    //     // This will have an normalizationo effect, which is ok for search.
    //     output.extend(c.to_lowercase());
    // }
    let stripped = hgk_strip_diacritics(text, 0xFFFFFFFF);
    output.push_str(&stripped);
}

impl<T: TokenStream> TokenStream for DiacriticTokenStream<'_, T> {
    fn advance(&mut self) -> bool {
        if !self.tail.advance() {
            return false;
        }
        // if self.token_mut().text.is_ascii() {
        //     // fast track for ascii.
        //     self.token_mut().text.make_ascii_lowercase();
        // } else {
        to_diacritic_free_unicode(&self.tail.token().text, self.buffer);
        mem::swap(&mut self.tail.token_mut().text, self.buffer);
        //}
        true
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ft_test() {
        let index_path = "../philologus-lex-loader/tantivy-datav4";
        let query = "example";
        let page = 1;

        let ft = FullTextWrapper::new(index_path);
        let res = ft.full_text_query(query, page, 20);

        assert_eq!(res.len(), 20);
    }

    #[test]
    fn ft_test_snippets() {
        let index_path = "../philologus-lex-loader/tantivy-datav4";
        let query = "fero";
        let page = 1;

        let ft = FullTextWrapper::new(index_path);
        let res = ft.full_text_snippets(query, page, 20);
        for r in &res {
            println!(
                "Snippet: {}: {}, {}, {}",
                r.word_id, r.lemma, r.lexicon, r.snippet
            );
        }
        assert_eq!(res.len(), 20);
    }
}
