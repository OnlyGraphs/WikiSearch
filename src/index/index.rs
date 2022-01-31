use std::collections::HashMap;


enum IndexEncoding {
    none,
    delta_encoding,
    elias_gamma_code,
}

//TODO: 
//Make sure you check for integer overflows. Or, implementing Delta encoding would mitigate any such problems.
#[derive(Debug)]
struct Index{
    postings: HashMap<String,  Vec<(i16,i16)>>,
    doc_freq: HashMap<String, u16>,
    
}



impl Index{

        
    fn add_posting(&mut self, token: String, docid: i16, word_pos: i16){
        
        let docid_and_word_pos_tuple = (docid, word_pos);
        self.postings.entry(token).or_insert(Vec::<(i16,i16)>::new()).push(docid_and_word_pos_tuple);

    }

    fn populate_index(&mut self, text: String){
        let mut split = text.split(" ");
        let mut word_pos = 0;
        let mut docid = 1;
        for token in split{
            self.add_posting(token.to_string(), docid, word_pos );
            *self.doc_freq.entry(token.to_string()).or_insert(0) +=1;
            word_pos+=1;
        }
    }

}



fn main(){

    let mut index = Index{
        postings: HashMap::new(),
        doc_freq: HashMap::new(),
    };


    index.populate_index("Hello world It is time to cry world".to_string());
    println!("{:?}", index);

}

// fn main(){
//     let mut map = HashMap::new();
//     let vec = Vec::<(i16,i16)>::new();
//     let mut m =  Index{
//         postings: map,
//     }; 
    
//     m.postings.entry("Hello".to_string()).or_insert(Vec::<(i16,i16)>::new()).push((3,3));
//     m.postings.get_mut("Hello").unwrap().push((4,5));
//     m.postings.get_mut("Hello").unwrap().push((4,5));

//     // *my_map.get_mut("a").unwrap() += 10;

//     // *m.postings.entry("Hello".to_string()).or_insert(vec);
//     println!("{:?}", m);
// }