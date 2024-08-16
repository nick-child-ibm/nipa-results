
use std::{env, fs, io, path::Path, cmp::Ordering, ffi::OsString };
use colored::{ColoredString, Colorize};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand},
    Terminal
};


mod test;
use test::Test;
mod tui;
use tui::App;

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
            println!("{}:", Colorize::bold(t.test.as_str()));
            prev_test = Some(&t.test);
        }

        let res_str : ColoredString;
        if t.result != 0 {
            res_str = Colorize::red(t.result.to_string().as_str());
            fail += 1;
        } else {
            res_str = Colorize::green(t.result.to_string().as_str());
            pass += 1;
        }
        println!("\t\t{}\t\t\t{}", t.patch, res_str);
    }
    println!("==============================================");
    println!("{}: {},\t{}: {},\t{}: {}", Colorize::bold("TOTAL"), pass+fail, Colorize::bold("Pass").green(), pass, Colorize::bold("FAIL").red(), fail);

}

fn tui_results (results: &mut Vec<Test>, p: String) -> io::Result<()> {

    let mut app =  App::from_results(results, &p);
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    app.run(terminal)?;

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn main() {
    let path;
    let mut tty = true;
    if env::args_os().len() < 2 || env::args_os().len() > 3 {
        println!("USAGE: {} [--stdout] <path to Nipa results>", env::args_os().next().unwrap().into_string().unwrap());
        return;
    }
    else if env::args_os().len() == 2 {
        path = env::args_os().last().unwrap().into_string().unwrap();
    }
    else {
       path =  env::args_os().into_iter().filter_map(
        |x| {
                                if *x == OsString::from("--stdout") {
                                    tty = false;
                                    None
                                }
                                else {
                                    Some(x)
                                }
        }).collect::<Vec<_>>().last().unwrap().clone().into_string().unwrap();
    }
 
    //path = env::args_os().last().unwrap().into_string().unwrap();
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
    if tty {
        let _ = tui_results(results, path);
    }
    else {
        print_results(results);
    }
}
