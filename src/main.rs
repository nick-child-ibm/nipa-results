
use std::{env, fs, io, path::Path, cmp::Ordering };
use colored::Colorize;

struct Test {
    patch: u8,
    result: i64,
    test: String
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

fn parse_results(path: &String, results: &mut Vec<Test>) -> io::Result<usize> {
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    for p in entries {
        if p.file_name().unwrap().to_str().unwrap() == String::from("retcode") {        
            let m = path.as_str();
            results.push(Test::new(Path::new(&m)));
        }
        else if p.is_dir() {
            parse_results(&String::from(p.to_str().unwrap()), results)?;
        }
    }
    return Ok(results.len())
}

fn print_results(results: &mut Vec<Test>) {
    let mut pass = 0; let mut fail = 0;
    // sort tests by test name + patch #
    results.sort_by(|a, b| if a.test.cmp(&b.test) == Ordering::Equal {a.patch.cmp(&b.patch)} else {a.test.cmp(&b.test)});

    let mut prev_test : Option<&String> = None;

    for t in results.iter() {
        if Some(&t.test) != prev_test {
            println!("{}:", t.test.bold());
            prev_test = Some(&t.test);
        }

        let res_str;
        if t.result != 0 {
            res_str = t.result.to_string().red();
            fail += 1;
        } else {
            res_str = t.result.to_string().green();
            pass += 1;
        }
        println!("\t\t{}\t\t\t{}", t.patch, res_str);
    }
    println!("==============================================");
    println!("{}: {},\t{}: {},\t{}: {}", "TOTAL".bold(), pass+fail, "PASS".bold().green(), pass, "FAIL".bold().red(), fail);

}

fn main() {
    if env::args_os().len() != 2 {
        println!("USAGE: {} <path to Nipa results>", env::args_os().next().unwrap().into_string().unwrap());
        return;
    }

    let path = env::args_os().last().unwrap().into_string().unwrap();
    let results = &mut Vec::<Test>::new();
    let parse_rc = parse_results(&path, results);
    let total_tests;
    match parse_rc {
        Ok(tot) => total_tests = tot,
        Err(e) => {
                            println!("Error while parsing result dir: {}", e);
                            return;
                        }
    };
    println!("found {} total tests...", total_tests);
    print_results(results);
}
