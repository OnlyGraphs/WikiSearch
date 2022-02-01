// The preprocess function expects to receive queries in the format x,y,z,p,w,..... E.g. #DIST,3,big whale,bunny
pub fn preprocess_query_str(query: str){
    // First split on comma
    let mut split_str = query.split(",")
    println!(split_str)
}