
use crate::model::record::Record;
use regex::Regex;
use bitflags::bitflags;

bitflags! {
    struct Scope: u16 { 
        /* INDIVIDUAL FIELDS */
        const METHOD =              0b0000000000000001;
        const SCHEME =              0b0000000000000010;
        const HOST =                0b0000000000000100;
        const PATH =                0b0000000000001000;
        const QUERY =               0b0000000000010000;
        const REQUEST_HEADERS =     0b0000000000100000;
        const REQUEST_BODY =        0b0000000001000000;
        const STATUS =              0b0000000010000000;
        const RESPONSE_HEADERS =    0b0000000100000000;
        const RESPONSE_BODY =       0b0000001000000000;

        /* CONVENIENCE UNIONS */
        const URL =                 0b0000000000011110;
        const HEADERS =             0b0000000100100000;
        const RESPONSE =            0b0000001110000000;
        const REQUEST =             0b0000000001111111;
        const ALL =                 0b0000001111111111;

        /* EXTRA CONTROLS ? */
      // const REVERSE =            0b1000000000000000;
      // const REPLACE =            0b0100000000000000;
      // const REPLACE_ALL =        0b0110000000000000;
      // const EXECUTE =            0b0001000000000000;
    }
}

// How to handle different follow-up actions?
enum Action {
    Replace,
    ReplaceAll,
    Execute
}

pub struct Rule {
    scope : Scope, // Where are we looking through? Everything, or just some fields? +/- matching?
    search : Regex, // What are we looking for?
    action : Option<Action>, // How do we act on a rule?
}

impl Rule {
    
    async fn new(search : Regex) -> Self {
        Self {
            scope : Scope::ALL,
            search,
            action : None,
        }
    }

    pub async fn check_string(&self, input : String) -> Result<(), ()> {
        match self.search.is_match(&input) {
            true => {
                return Ok(())
            },
            false => {
                return Err(())
            },
        }
    }

    pub async fn apply(&self, mut record: Record) -> Option<Record> {

        let mut source = self.get_scoped(&record).await.unwrap();

        match self.search.is_match(&source) {
            true => {
                return Some(record)
            },
            false => {
                return None 
            },
        }
    }

    pub async fn get_scoped(&self, record : &Record) -> Option<String> {
        let mut input : String = String::new();
        if Scope::is_empty(&self.scope) {
            return None
        }
        if (Scope::METHOD & self.scope) == Scope::METHOD {
            input.push_str(&record.method);
        }
        if (Scope::SCHEME & self.scope) == Scope::SCHEME {
            input.push_str(&record.scheme);
        }
        if (Scope::HOST & self.scope) == Scope::HOST {
            input.push_str(&record.host);
        }
        if (Scope::PATH & self.scope) == Scope::PATH {
            input.push_str(&record.path);
        }
        if (Scope::QUERY & self.scope) == Scope::PATH {
            input.push_str(&record.query);
        }
        if (Scope::REQUEST_HEADERS & self.scope) == Scope::REQUEST_HEADERS {
            for (name, val) in &record.request_headers {
                input.push_str(&(name.to_owned() + &": ".to_string() + &val));
            }
        }
        if (Scope::REQUEST_BODY & self.scope) == Scope::REQUEST_BODY {

        }
        if (Scope::RESPONSE_HEADERS & self.scope) == Scope::RESPONSE_HEADERS {
            for (name, val) in &record.response_headers {
                input.push_str(&(name.to_owned() + &": ".to_string() + &val));
            }
        }
        if (Scope::RESPONSE_BODY & self.scope) == Scope::RESPONSE_BODY {

        }
        if (Scope::STATUS & self.scope) == Scope::STATUS {
            input.push_str(&record.status.to_string());
        }
        match input.is_empty() {
            true => None,
            false => Some(input),
        }
    }

}

pub struct FilterChain {
    filter_chain : Vec<Rule>,
}

impl FilterChain {
    
    async fn new() -> Self {
        Self {
            filter_chain : Vec::<Rule>::new(), 
        }
    }

    // You need to add logging too.
    pub async fn filter(&mut self, record: Record) -> Option<Record> {
        for rule in self.filter_chain.iter() {
            match rule.check_string(record.to_string()).await {
                Ok(_) => { },
                Err(_) => { return None },
            }
        }
        return Some(record)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    // Set union -> '|'.
    // Set difference -> '-'.
    //
    // https://docs.rs/regex/latest/regex/#syntax
    // [xyz]         A character class matching either x, y or z (union).
    // [^xyz]        A character class matching any character except x, y and z.
    // [a-z]         A character class matching any character in range a-z.
    // [[:alpha:]]   ASCII character class ([A-Za-z])
    // [[:^alpha:]]  Negated ASCII character class ([^A-Za-z])
    // [x[^xyz]]     Nested/grouping character class (matching any character except y and z)
    // [a-y&&xyz]    Intersection (matching x or y)
    // [0-9&&[^4]]   Subtraction using intersection and negation (matching 0-9 except 4)
    // [0-9--4]      Direct subtraction (matching 0-9 except 4)
    // [a-g~~b-h]    Symmetric difference (matching `a` and `h` only)
    // [\[\]]        Escaping in character classes (matching [ or ])


    #[test]
    fn scope_equivalence() {
        assert_eq!(Scope::ALL, (Scope::REQUEST | Scope::RESPONSE)); // Set union.
        assert_eq!(Scope::HEADERS, (Scope::REQUEST_HEADERS | Scope::RESPONSE_HEADERS));
        assert_eq!(Scope::ALL, (Scope::ALL - Scope::empty())); // Set difference.
        assert_eq!(Scope::REQUEST, (Scope::REQUEST_HEADERS | Scope::REQUEST_BODY | Scope::URL | Scope::METHOD));
    }

    #[tokio::test]
    async fn rule_match() {
        let mut rule = Rule::new(Regex::new("[[:alpha:]]").unwrap()).await;
        let result = rule.check_string("FOOBAR".to_string()).await;
        match result {
            Ok(_) => { assert!(true); },
            Err(_) => { panic!("FOOBAR didn't match alpha."); },
        }
        let result = rule.check_string("123456789".to_string()).await;
        match result {
            Ok(_) => { panic!("123456789 matched alpha!"); },
            Err(_) => { assert!(true); },
        }
    }

    #[tokio::test]
    async fn find_action() {

    }

    #[test]
    fn replace_action() {

    }

    // You need to parse and abstract session tokens.
    // You need to limit maximum body size. (?)
    // You need to remove unnecessary headers.
    // You need to check for uniqueness of records to avoid duplication.

    // You need to support interoperability or configs to define your own rules.
    // You need to store 'traffic records' in a Vec inside 'endpoint records'.
    //          Traffic records: As-is with no transformations.
    //          Endpoint records: Abstracted minimally viable product with generics.
}
