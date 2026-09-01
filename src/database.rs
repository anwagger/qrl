/*
Stores data for link redirects in a directory-based Trie

the data will be in data.json, with fields:
{
    "user":andrew;
    "url":https://andrew.wagger.net,
    "time":TIME_CREATED_UTC,
}

*/

use std::fs;

use std::cmp::min;

use std::path::PathBuf;
use std::path::Path;

use json;

use json::JsonValue;
use json::object;

pub struct Record {
    pub value: Box<String>,
    pub user: Box<String>,
    pub time: u64,
}

impl Record {
    pub fn new (value : &str, user : &str, time : u64) -> Record{
        return Record{
            value : Box::new(value.to_string()),
            user : Box::new(user.to_string()),
            time : time,
        }
    }
    pub fn from_json(value: JsonValue) -> Option<Record> {

        return Some(Record{
            value: match &value["value"].as_str(){
                Some(s) => Box::new(s.to_string()),
                _ => {return None}
            },
            user: match &value["user"].as_str(){
                Some(s) => Box::new(s.to_string()),
                _ => {return None}
            },
            time: match &value["time"].as_u64(){
                Some(n) => *n,
                _ => return None
            },
        });
    }
    pub fn to_json(&self) -> JsonValue {
        let mut json_data = object!{};
        json_data["value"] = JsonValue::String(*self.value.clone());
        json_data["user"] = JsonValue::String(*self.user.clone());
        json_data["time"] = self.time.into();
        return json_data;
    }
}

pub fn initialize() -> Result<bool,()> {
    return create_dir_if_not(Path::new("./data"));
}

fn create_dir_if_not(path : &Path) -> Result<bool,()> {
    if let Ok(_) = fs::read_dir(path) {
        return Ok(false);
    }else{
        if let Ok(_) = fs::create_dir_all(path) {
            return Ok(true)
        }else{
            return Err(());
        }
    }
}

pub fn set_record(key: &str, value: Record) -> Result<bool,&str>{
    // traverse down to where the record should be
    let (path,left) = match traverse(PathBuf::from("./data"),key) {
        Err(e) => return Err(e),
        Ok(r) => r
    };
    // might be able to combine these two?
    if left.len() == 0 {
        return match fs::write(path.join("data.json"),json::stringify(value.to_json())){
          Err(_) => Err("Got JSON stringify error"),
          Ok(_) => Ok(true)
        };
    }
    // need to check for splitting directory!
    let files = match get_files(&path.as_path()) {
        Err(_) => {return Err("Get Files Error")},
        Ok(f) => f
    };

    for file_buf in files {
        let file = file_buf.as_path();
        let name: &str = match file.file_name(){
            None => return Err("Couldn't get file name?"),
            Some(os_str) => match os_str.to_str() {
                Some(s) => s,
                None => return Err("Couldn't convert file name?"),
            }
        };
        let prefix = find_shared_prefix(name,left);
        if prefix > 0 {
            // move child down
            let (pre,rem) = name.split_at(prefix);
            // Move child

            let _ = create_dir_if_not(&path.join(pre).join(rem));
            let _  = fs::rename(file,path.join(pre).join(rem));
            
            // nothing left, so delete self
            let (pre,rem) = left.split_at(prefix);
            let new_path = path.join(pre).join(rem);

            let _ = create_dir_if_not(&new_path);
            return match fs::write(new_path.join("data.json"),json::stringify(value.to_json())){
                Err(e) => 
                {println!("{}",e);Err("Got JSON stringify error")
                },
                Ok(_) => Ok(true)
            };    
        }
    };    
    let new_path = path.join(left);

    // make new path
    let _ = create_dir_if_not(&new_path);
    return match fs::write(new_path.join("data.json"),json::stringify(value.to_json())){
        Err(_) => Err("Got JSON stringify error"),
        Ok(_) => Ok(true)
    };  
    
}


pub fn remove_record(key: &str) -> Result<bool,&str>{
    // traverse down to where the record should be
    let (path,left) = match traverse(PathBuf::from("./data"),key) {
        Err(e) => return Err(e),
        Ok(r) => r
    };
    // if the directories don't go to it, it doesn't exist
    if left.len() > 0 {
        return Ok(false);
    }
    // check if a record exists
    let mut has_data = false;
    // count number of database children
    let mut n_children = 0;
    let mut last_child = PathBuf::from("");

    let files = match get_files(&path.as_path()) {
        Err(_) => {return Err("Get files Error")},
        Ok(f) => f
    };

    for file_buf in files {
        let file = file_buf.as_path();
        if file.is_dir() {
            n_children += 1;
            last_child = file_buf;
        }else{
            let name: &str = match file.file_name(){
                None => return Err("Couldn't get file name?"),
                Some(os_str) => match os_str.to_str() {
                    Some(s) => s,
                    None => return Err("Couldn't convert file name?"),
                }
            };
            if name == "data.json" {
                match fs::remove_file(file) {
                    Err(_) => return Err("Got remove_file error"),
                    Ok(_) => {
                        has_data = true;
                    }
                };
            }
        }
    }
    
    // use child count to clean up Trie
    if n_children == 0 {
        // nothing left, so delete self
        match fs::remove_dir(&path) {
            Err(_) => return Err("Directory most likely not empty, maybe use remove_dir_all?"),
            Ok(_) => {}
        };
        let parent: &Path = match path.parent() {
                Some(p) => p,
                None => return Err("Can't go back to merge paths!")
            };
        match check_for_merge(PathBuf::from(parent)) {
            Err(e) => println!("MERGE ERR {}",e),
            Ok(_) => {}
        }

    }else if n_children == 1 {
        let name: &str = match path.file_name(){
                None => return Err("Couldn't get file name?"),
                Some(os_str) => match os_str.to_str() {
                    Some(s) => s,
                    None => return Err("Couldn't convert file name?"),
                }
        };
        // if we can go back
        if path.to_str() != Some("./data") {
            let parent: &Path = match path.parent() {
                Some(p) => p,
                None => return Err("Can't go back to merge paths!")
            };
            // move child up
            let _ = fs::rename(last_child.as_path(),parent.join(name.to_owned()+left));
            // nothing left, so delete self
            let _  = check_for_merge(PathBuf::from(parent));

            match fs::remove_dir(path) {
                Err(_) => return Err("Directory most likely not empty, maybe use remove_dir_all?"),
                Ok(_) => {}
            }
        }
    }
    

    // no data, didn't delete
    return Ok(has_data);
}

pub fn get_record(key: &str) -> Result<Option<Record>,&str>{
    let (path,left) = match traverse(PathBuf::from("./data/"),key) {
        Err(e) => return Err(e),
        Ok(r) => r
    };
    if left.len() > 0 {
        return Ok(None);
    }

    return match path.join("data.json").to_str() {
        None => Err("Can't parse path?"),
        Some(p) => {
            let file_data = match fs::read_to_string(p) {
                Ok(s) => s,
                Err(_) => "?".to_string()
            };
            return match json::parse(&file_data) {
                Err(_) => Err("Malformed JSON"),
                Ok(j) => Ok(Record::from_json(j))
            }
        }
    }

}

fn traverse<'a>(path_buf: PathBuf, key: &'a str) -> Result<(PathBuf,&'a str),&'a str>{
    if key.len() == 0 {
        return Ok((path_buf,key));
    }
    let files = match get_files(&path_buf.as_path()) {
        Err(_) => {return Err("Get Files Error")},
        Ok(f) => f
    };
    

    for file_buf in files {
        let file = file_buf.as_path();
        if file.is_dir() {
            let name: &str = match file.file_name(){
                None => return Err("Couldn't get file name?"),
                Some(os_str) => match os_str.to_str() {
                    Some(s) => s,
                    None => return Err("Couldn't convert file name?"),
                }
            };
            // let re = match Regex::new(format!("^{}",name));
            let prefix = find_shared_prefix(name,key);
            if prefix == name.len() {
            // if regex!(re_str).is_match(key) {
                let (_,left) = key.split_at(prefix);
                return traverse(file.to_path_buf(),left);
            }
        }
    }
    return Ok((path_buf,key));
}

fn find_shared_prefix(a: &str,b: &str) -> usize {
    let min_len = min(a.len(),b.len());
    let mut prefix_len = 0;
    let mut a_bytes = a.bytes();
    let mut b_bytes = b.bytes();
    for _ in 0..min_len {
        let a_char = a_bytes.next();
        let b_char = b_bytes.next();
        if a_char != b_char {
            return prefix_len;
        }
        prefix_len += 1;
    };
    return prefix_len;
}

fn get_files(path: &Path) -> Result<Box<dyn Iterator<Item = PathBuf>>,&str> {
    return match fs::read_dir(path) {
        Err(_) =>  {
            println!("CANT READ PATH {}",path.display());
            return Err("Got read_dir error")},
        Ok(r) => Ok(Box::new(r.filter_map(|e| -> Option<PathBuf> 
            {
                match e {
                    Err(_) => None,
                    Ok(d) => Some(d.path())
                }
            }
        )))
    };
}

fn check_for_merge(path: PathBuf) -> Result<bool,&'static str>{
    let mut n_children = 0;
    let mut last_child = PathBuf::from("");

    let files = match get_files(&path.as_path()) {
        Err(_) => {return Err("Get files Error")},
        Ok(f) => f
    };

    for file_buf in files {
        let file = file_buf.as_path();
        if file.is_dir() {
            n_children += 1;
            last_child = file_buf;
        }
    }
    // use child count to clean up Trie
    if n_children == 1 {
        let name: &str = match path.file_name(){
                None => return Err("Couldn't get file name?"),
                Some(os_str) => match os_str.to_str() {
                    Some(s) => s,
                    None => return Err("Couldn't convert file name?"),
                }
        };
        // if we can go back
        if path.to_str() != Some("./data") {
            let parent: &Path = match path.parent() {
                Some(p) => p,
                None => return Err("Can't go back to merge paths!")
            };
            // move child up
            let name2 : &str = match last_child.file_name(){
                None => return Err("Couldn't get file name?"),
                Some(os_str) => match os_str.to_str() {
                    Some(s) => s,
                    None => return Err("Couldn't convert file name?"),
                }
            };
            match fs::rename(last_child.as_path(),parent.join(name.to_owned()+name2)) {
                Err(e) => println!("rname ERR {}",e),
                Ok(_) => {}
            };
            // nothing left, so delete self
            match fs::remove_dir(&path) {
                Err(_) => return Err("Directory most likely not empty, maybe use remove_dir_all?"),
                Ok(_) => {}
            }

            //return check_for_merge(PathBuf::from(parent));
        }
        return Ok(true);
    }
    return Ok(false);
}

pub fn dump_db(){
    dump(PathBuf::from("./data"));
}

fn dump(path_buf : PathBuf){
    let files = match get_files(&path_buf.as_path()) {
        Err(_) => {return},
        Ok(f) => f
    };
    

    for file_buf in files {
        let file = file_buf.as_path();
        if file.is_dir() {
            match file.to_str() {
                None => {},
                Some(p) => println!("DIR: {}",p),
            }
            let _ = dump(file_buf);
        }else{
            let file_path = match file.to_str() {
                None => "",
                Some(p) => p
            };
            let file_data = match fs::read_to_string(file_path) {
                Ok(s) => s,
                Err(_) => "?".to_string()
            };
            match json::parse(&file_data) {
                    Err(e) => {println!("PARSE ISSUE {}",e)},
                    Ok(j) => println!("DATA: {},",json::stringify(j)),
                };
            
        }
    }
}