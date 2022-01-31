// extern crate chrono;

use std::collections::HashMap;
// use chrono::{DateTime, NaiveDate};

#[derive(Debug)]
enum Domain{
    org,
}



enum IndexEncoding {
    none,
    delta_encoding,
    elias_gamma_code,
}

//TODO: 
//Make sure you check for integer overflows. Or, implementing Delta encoding would mitigate any such problems.
#[derive(Debug)]
struct BasicIndex{
    title: String,
    domain: Domain,
    // lastUpdatedDate: NaiveDate,
    postings: HashMap<String,  Vec<(i16,i16)>>,
    links: Vec<i16>, // List of tuples, where each element is: (Doc id, (Word_pos start, word_pos end))
    categories: Vec<String>,
    citation_positions:  Vec<(i16, (i16,i16))>,
    doc_freq: HashMap<String, u16>,

}



impl BasicIndex{

        
    fn add_posting(&mut self, token: String, docid: i16, word_pos: i16){
        
        let docid_and_word_pos_tuple = (docid, word_pos);
        self.postings.entry(token).or_insert(Vec::<(i16,i16)>::new()).push(docid_and_word_pos_tuple);

    }

    fn add_document(&mut self, text: String, doc_id: i16,){
        let mut split = text.split(" ");
        let mut word_pos = 0;
        for token in split{
            self.add_posting(token.to_string(), doc_id, word_pos );
            *self.doc_freq.entry(token.to_string()).or_insert(0) +=1;
            word_pos+=1;
        }
    }

    fn populate_index(&mut self, text: String, doc_id: i16, links: Vec<String>, categories: Vec<String>, citation_ids: Vec<i16>){

    }

    


}

fn parse_from_date(){
    
}

fn main(){

    let mut index = BasicIndex{
        title:  String::from("Hello World"),
        postings: HashMap::new(),
        links: Vec::<i16>::new(),
        citation_positions: Vec::<(i16, (i16,i16))>::new(),
        doc_freq: HashMap:: new(),
        categories: Vec::new(),
        domain: Domain::org,
        // lastUpdatedDate: DateTime::parse_from_str("2000-06-29", "%Y-%m-%d").unwrap(),
    };


    index.add_document("Hello world It is time to cry world".to_string(), 1);
    println!("{:?}", index);

}

