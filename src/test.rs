use std::{fs, path::Path};
pub struct Test {
   pub patch: u8,
   pub result: i64,
   pub test: String,
}
impl Clone for Test {
    fn clone (&self) -> Self {
        return Test{patch: self.patch, result: self.result, test: self.test.clone() }
    }
}
impl Test {
    // assumes retcode exists at <p>/retcode 
    pub fn new(p: & Path) -> Test {
       // test name is parent directory of the retcode file
       let test_name =  String::from(p.file_name().unwrap().to_str().unwrap());
       // patch number is the parent directory of the test name directory
       // use 0 if test is not related to a patch
       let patch_id: u8;
       match p.parent() {
        Some(parnt) => {
            let  num = String::from(parnt.file_name().unwrap().to_str().unwrap());
            match num.parse::<u8>() {
                Ok(num) => patch_id = num,
                Err(_) => patch_id = 0
            }},
        None => patch_id = 0
       };
       let mut pb = p.to_path_buf(); 
       pb.push("retcode");

       let str_rc = fs::read_to_string(pb).unwrap();
       let retcode = str_rc.parse::<i64>().unwrap_or(404);
       Test {patch: patch_id, result: retcode, test: test_name}
    }
}
