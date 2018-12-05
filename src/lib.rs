use combine::{many1, none_of, Parser, sep_by, sep_end_by, choice, attempt};
use combine::char::*;

use std::result::*;
use combine::error::*;
use std::string::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;

#[derive(Debug)]
#[derive(Clone)]
struct Group {
    name: String,
    password: String,
    gid: u32,
    users: Vec<String>,
}

fn parse_fields(fields: Vec<String>) -> Result<Group, StringStreamError> {
    let userp = many1(none_of(",".chars()));
    let mut parser = sep_by(userp, char(',')).map(|words: Vec<String> | words);
    let ret = match fields.as_slice() {
        [username, password, gidstring, users_fields] => {
            let gid = gidstring.parse::<u32>().expect("gid must be an positive integer");
            parser.parse(users_fields.as_str()).map(|x| Group {name: username.clone(), password: password.clone(), gid: gid, users: x.0})
        },
        [username, password, gidstring] => {
            let gid = gidstring.parse::<u32>().expect("gid must be an positive integer");
            Ok(Group {name: username.clone(), password: password.clone(), gid: gid, users: Vec::new() })
        },
        _ => Err(StringStreamError::UnexpectedParse)
        };
    ret
}

fn parse_group_entry (line: &str) -> Result<Group, StringStreamError> {
    let word = many1(none_of(":".chars()));
    let grp_with_users = sep_by(word.clone(), char(':')).map(|words: Vec<String> | words);
    let grp_without_users = sep_end_by(word.clone(), char(':')).map(|words: Vec<String> | words);
    let mut p = choice((attempt(grp_with_users), grp_without_users));
    p.parse(line).map(|x| x.0).map(|x| parse_fields(x)).and_then(|x| x)
}

pub fn from_group(desired: &str) -> Result<u32, std::io::Error> {
    let mut f = File::open("/etc/group")?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    let all_groups :Result<Vec<Group>, StringStreamError> = contents.lines().map(|e| parse_group_entry(e)).collect();
    let groups: Vec<u32> = all_groups.expect("failed to parse /etc/group").iter().filter(|x| x.name == desired).map(|x|x.gid).collect();
    match groups[..] {
        [found] => Ok(found),
        _  => {
            let errmsg = String::from("cannot find group: ");
            Err(Error::new(ErrorKind::Other, errmsg + desired))
        },
    }
}
