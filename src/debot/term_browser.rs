/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/
use crate::config::Config;
use crate::crypto::load_keypair;
use crate::helpers::load_ton_address;
use debot_engine::{BrowserCallbacks, DAction, DEngine, STATE_EXIT};
use std::cell::RefCell;
use std::io::{self, BufRead, Write};
use std::rc::Rc;
use ton_client_rs::{Ed25519KeyPair, TonAddress};

struct TerminalBrowser {
    state_id: u8,
    active_actions: Vec<DAction>,
    network: String,
}

impl TerminalBrowser {
    pub fn new(network: String) -> Self {
        Self {
            state_id: 0,
            active_actions: vec![],
            network,
        }
    }

    pub fn select_action(&self) -> Option<DAction> {
        if self.state_id == STATE_EXIT {
            return None;
        }
        if self.active_actions.len() == 0 {
            debug!("no more actions, exit loop");
            return None;
        }

        loop {
            let res = action_input(self.active_actions.len());
            if res.is_err() {
                println!("{}", res.unwrap_err());
                continue;
            }
            let (n, _, _) = res.unwrap();
            let act = self.active_actions.get(n - 1);
            if act.is_none() {
                println!("Invalid action. Try again.");
                continue;
            }
            return act.map(|a| a.clone());
        }
    }
}

struct Callbacks {
    browser: Rc<RefCell<TerminalBrowser>>,
}

impl Callbacks {
    pub fn new(browser: Rc<RefCell<TerminalBrowser>>) -> Self {
        Self { browser }
    }
}

impl BrowserCallbacks for Callbacks {
    /// Debot asks browser to print message to user
    fn log(&self, msg: String) {
        println!("{}", msg);
    }

    /// Debot is switched to another context.
    fn switch(&self, ctx_id: u8) {
        debug!("switched to ctx {}", ctx_id);
        self.browser.borrow_mut().state_id = ctx_id;
        if ctx_id == STATE_EXIT {
            println!("Debot shutdown");
            return;
        }

        self.browser.borrow_mut().active_actions = vec![];
    }

    /// Debot asks browser to show user an action from the context
    fn show_action(&self, act: DAction) {
        let mut browser = self.browser.borrow_mut();
        println!("{}) {}", browser.active_actions.len() + 1, act.desc);
        browser.active_actions.push(act);
    }

    // Debot engine asks user to enter argument for an action.
    fn input(&self, prefix: &str, value: &mut String) {
        let stdio = io::stdin();
        let mut reader = stdio.lock();    
        let mut writer = io::stdout();
        *value = input(prefix, &mut reader, &mut writer);
    }

    /// Debot engine requests keys to sign something
    fn load_key(&self, keys: &mut Ed25519KeyPair) {
        let stdio = io::stdin();
        let mut reader = stdio.lock();    
        let mut writer = io::stdout();
        *keys = input_keys(&mut reader, &mut writer);
    }
    /// Debot asks to run action of another debot
    fn invoke_debot(&self, debot: TonAddress, action: DAction) -> Result<(), String> {
        debug!("fetching debot {} action {}", &debot, action.name);
        println!("invoking debot {}", &debot);
        let browser = Rc::new(
            RefCell::new(
                TerminalBrowser::new(self.browser.borrow().network.clone())
            )
        );
        let callbacks = Box::new(Callbacks::new(Rc::clone(&browser)));
        
        let mut debot = DEngine::new(debot, None, &self.browser.borrow().network, callbacks);
        debot.fetch()?;
        
        debot.execute_action(&action)?;
        loop {
            let action = browser.borrow().select_action();
            match action {
                Some(act) => debot.execute_action(&act)?,
                None => break,
            }
        }
        Ok(())
    }
}

fn input<R, W>(prefix: &str, reader: &mut R, writer: &mut W) -> String
where
    R: BufRead,
    W: Write,    
{
    let mut input_str = "".to_owned();
    let mut argc = 0;
    while argc == 0 {
        print!("{} > ", prefix);
        if let Err(e) = writer.flush() {
            println!("failed to flush: {}", e);
            return input_str;
        }
        if let Err(e) = reader.read_line(&mut input_str) {
            println!("failed to read line: {}", e);
            return input_str;
        }
        argc = input_str
            .split_whitespace()
            .map(|x| x.parse::<String>().unwrap())
            .collect::<Vec<String>>()
            .len();
    }
    input_str.trim().to_owned()
} 
    

fn action_input(max: usize) -> Result<(usize, usize, Vec<String>), String> {
    let mut a_str = String::new();
    let mut argc = 0;
    let mut argv = vec![];
    println!();
    while argc == 0 {
        print!("debash$ ");
        let _ = io::stdout().flush();
        io::stdin()
            .read_line(&mut a_str)
            .map_err(|e| format!("failed to read line: {}", e))?;
        argv = a_str
            .split_whitespace()
            .map(|x| x.parse::<String>().expect("parse error"))
            .collect::<Vec<String>>();
        argc = argv.len();
    }
    let n = usize::from_str_radix(&argv[0], 10)
        .map_err(|_| format!("Oops! Invalid action. Try again, please."))?;
    if n > max {
        Err(format!("Auch! Invalid action. Try again, please."))?;
    }

    Ok((n, argc, argv))
}

fn input_keys<R, W>(reader: &mut R, writer: &mut W) -> Ed25519KeyPair
where
    R: BufRead,
    W: Write,  
{
    let enter_str = "enter seed phrase or path to keypair file";
    let mut value = input(enter_str, reader, writer);

    let mut pair = load_keypair(&value);
    while let Err(e) = pair {
        println!("Invalid keys: {}. Try again.", e);
        value = input(enter_str, reader, writer);
        pair = load_keypair(&value);
    }
    pair.unwrap()
}

pub fn run_debot_browser(addr: &str, config: Config) -> Result<(), String> {
    let url = config.url.clone();
    let browser = Rc::new(RefCell::new(TerminalBrowser::new(config.url)));

    let callbacks = Box::new(Callbacks::new(Rc::clone(&browser)));
    let mut debot = DEngine::new(load_ton_address(addr)?, None, &url, callbacks);
    debot.start()?;

    loop {
        let action = browser.borrow().select_action();
        match action {
            Some(act) => debot.execute_action(&act)?,
            None => break,
        }
    }
    Ok(())
}

#[cfg(test)]
mod browser_tests {
    use super::*;
    use std::fs::File;

    const PUBLIC: &'static str = "9711a04f0b19474272bc7bae5472a8fbbb6ef71ce9c193f5ec3f5af808069a41";
    const PRIVATE: &'static str = "cdf2a820517fa783b9b6094d15e650af92d485084ab217fc2c859f02d49623f3";
    const SEED: &'static str = "episode polar pistol excite essence van cover fox visual gown yellow minute";
    const KEYS_FILE: &'static str = "./keys.json";

    fn create_keypair_file(name: &str) {
        let mut file = File::create(name).unwrap();
        file.write_all(format!(r#"{{
            "public": "{}",
            "secret": "{}"
        }}"#, PUBLIC, PRIVATE).as_bytes()).unwrap();
    }

    #[test]
    fn load_key_from_file() {
        let mut in_data = KEYS_FILE.as_bytes();
        let mut out_data = vec![];

        create_keypair_file(KEYS_FILE);
        let keys = input_keys(&mut in_data, &mut out_data);
        
        assert_eq!(format!("{}", keys.public), PUBLIC);
        assert_eq!(format!("{}", keys.secret), PRIVATE);        
    }

    #[test]
    fn load_key_from_seed() {
        let mut in_data = SEED.as_bytes();
        let mut out_data = vec![];

        let keys = input_keys(&mut in_data, &mut out_data);
        
        assert_eq!(format!("{}", keys.public), PUBLIC);
        assert_eq!(format!("{}", keys.secret), PRIVATE);
    }
}