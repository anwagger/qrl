pub use qrl::database;
pub use qrl::database::Record;



fn main() {

    let _ = database::initialize();
    let _ = database::set_record("test",Record::new("andrew.wagger.net","Andrew", 500));
    let _ = database::set_record("teamwork",Record::new("marc.wagger.net","Andrew", 600));
    let _ = database::set_record("dreamwork",Record::new("eric.wagger.net","Andrew", 700));
    let _ = database::set_record("teams",Record::new("david.wagger.net","Andrew", 800));

    match database::get_record("teams") {
        Err(e) => {println!("{}",e);},
        Ok(opt) => match opt {
            None => {println!("Not Found");},
            Some(r) => {println!("Found: {}",r.value);},
        }
    }
    match database::get_record("dreamwork") {
        Err(e) => {println!("{}",e);},
        Ok(opt) => match opt {
            None => {println!("Not Found");},
            Some(r) => {println!("Found: {}",r.value);},
        }
    }
    println!();

    database::dump_db();
    println!();
    let _ = database::remove_record("test");

    database::dump_db();
    println!();

    let _ = database::remove_record("teams");
    database::dump_db();

}
