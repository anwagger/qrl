/*
Stores data for link redirects in a directory-based Trie

the data will be in data.json, with fields:
{
    "user":andrew;
    "url":https://andrew.wagger.net,
    "time":TIME_CREATED_UTC,
}

*/

use regex::Regex;

use std::io;
use std::fs;


use json;
use json::JsonValue;

pub struct Record {
    value: str,
    user: str,
    time: u64,
}

impl Record {
    fromJson(value: JsonValue) -> Option<Record> {
        return Record{
            value: match value["value"]{
                String(s) => s.to_str();
                _ => return None
            };
            user = match value["user"]{
                String(s) => s.to_str();
                _ => return None
            };
            time = match value["time"]{
                Number(n) => n;
                _ => return None
            };
        };
    };
    toJson(&self) -> JsonValue {
        return object!{
            value: self.value;
            user: self.user;
            time: self.time;
        }
    };
}

pub Result<bool> fn set_record(key: str, value: Record){
    // traverse down to where the record should be
    let (path,left) = match traverse(Path::new("./data")) {
        Err(e) => return Err(e);
        Ok(r) => r;
    }
    // might be able to combine these two?
    if left.len() == 0 {
        match fs::write(path.join("data.json"),json::stringify(value.toJson())){
          Err(e) => return Err(e);
          Ok(_) => Ok(true);  
        }
    }else{
        match fs::write(path.join(left).join("data.json"),json::stringify(value.toJson())){
          Err(e) => return Err(e);
          Ok(_) => Ok(true);  
        }
    }

    return Ok(false);
}


pub Result<bool> fn remove_record(key: str){
    // traverse down to where the record should be
    let (path,left) = match traverse(Path::new("./data")) {
        Err(e) => return Err(e);
        Ok(r) => r;
    }
    // if the directories don't go to it, it doesn't exist
    if left.len() > 0 {
        return Ok(false);
    }
    // check if a record exists
    let hasData = false;
    // count number of database children
    let nChildren = 0;
    let lastChild = "";

    let files = match fs::read_dir(path) {
        Err(e) =>  return Err(e);
        Ok(r) => r.map(|e| e.path());
    }
    for file in files {
        
        if file.is_dir() {
            nChildren ++;
            lastChild = file;
        }else{
            let name = match file.file_name(){
                None => return Err("Couldn't get file name?");
                Some(osStr) => osStr.to_str();
            }
            if name == "data.json" {
                match fs::remove_file(file) {
                    Err(e) => return Err(e);
                    Ok(_) => {
                        hasData = true;
                    };
                }
            }
        }
    }
    
    // use child count to clean up Trie
    if nChildren == 0 {
        // nothing left, so delete self
        match fs::remove_dir(path) {
            Err(e) => return Err("Directory most likely not empty, maybe use remove_dir_all?");
            Ok(_) => {}
        }
    }else if nChildren == 1 {
        let name = match path.file_name(){
                None => return Err("Couldn't get file name?");
                Some(osStr) => osStr.to_str();
        }
        // if we can go back
        if path.to_str() != "./data" {
            let parent = match path.parent() {
                Some(p) => p;
                None => Err("Can't go back to merge paths!");
            };
            // move child up
            fs::rename(lastChild,parent.join(name+key));
            // nothing left, so delete self
            match fs::remove_dir(path) {
                Err(e) => return Err("Directory most likely not empty, maybe use remove_dir_all?");
                Ok(_) => {}
            }
        }
    }

    // no data, didn't delete
    return Ok(hasData);
}

pub Result<Option<Record>> fn get_record(str key){
    let (path,left) = match traverse(Path::new("./data/")) {
        Err(e) => return Err(e);
        Ok(r) => r;
    }
    if left.len() > 0 {
        return Ok(None);
    }

    return match json.parse(path.to_str()) {
        Err(e) => Ok(None);
        Ok(j) => Ok(Some(Record::fromJson(j)));
    }
}

Result<(Path,str)> fn traverse(path: Path, key: const str){
    if key.len() == 0 {
        return Ok((path,str))
    }
    let files = match fs::read_dir(path) {
        Err(e) =>  return Err(e);
        Ok(r) => r.map(|e| e.path());
    }

    for file in files {
        if file.is_dir() {
            let name = match file.file_name(){
                None => return Err("Couldn't get file name?");
                Some(osStr) => osStr.to_str();
            }
            if regex!(format!("^{}",name)).is_match(key) {
                let (used,left) = key.split_at(name.len())
                return traverse(file.join(used),left);
            }
        }
    }
    return Ok((path,key));
}

