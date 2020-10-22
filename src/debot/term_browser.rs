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
use crate::crypto::{generate_keypair_from_mnemonic, keypair_to_ed25519pair};
use crate::helpers::load_ton_address;
use debot_engine::{BrowserCallbacks, DAction, DEngine, STATE_EXIT};
use std::cell::RefCell;
use std::io::{self, Write};
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
        let mut input_str = "".to_owned();
        let mut argc = 0;
        while argc == 0 {
            print!("{} > ", prefix);
            let _ = io::stdout().flush();
            if let Err(e) = io::stdin().read_line(&mut input_str) {
                println!("failed to read line: {}", e)
            }
            argc = input_str
                .split_whitespace()
                .map(|x| x.parse::<String>().unwrap())
                .collect::<Vec<String>>()
                .len();
        }
        *value = input_str.trim().to_owned();
    }

    /// Debot engine requests keys to sign something
    fn load_key(&self, keys: &mut Ed25519KeyPair) {
        let mut value = String::new();
        self.input("enter seed phrase", &mut value);

        let mut pair = generate_keypair_from_mnemonic(&value);
        while let Err(_) = pair {
            println!("Invalid seed phrase. Try again");
            self.input("enter seed phrase", &mut value);
            pair = generate_keypair_from_mnemonic(&value);
        }
        *keys = keypair_to_ed25519pair(pair.unwrap()).unwrap();
    }
    /// Debot asks to run action of another debot
    fn invoke_debot(&self, debot: TonAddress, action: DAction) -> Result<(), String> {
        debug!("fetching debot {} action {}", &debot, action.name);
        println!("invoking debot {}", &debot);
        let callbacks = Box::new(Callbacks::new(Rc::clone(&self.browser)));
        let mut debot = DEngine::new(debot, None, &self.browser.borrow().network, callbacks);
        debot.fetch()?;
        debot.execute_action(&action)?;
        loop {
            let action = self.browser.borrow().select_action();
            match action {
                Some(act) => debot.execute_action(&act)?,
                None => break,
            }
        }
        Ok(())
    }
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

pub fn run_debot_browser(addr: &str, abi: Option<String>, config: Config) -> Result<(), String> {
    let url = config.url.clone();
    let browser = Rc::new(RefCell::new(TerminalBrowser::new(config.url)));

    let callbacks = Box::new(Callbacks::new(Rc::clone(&browser)));
    let mut debot = DEngine::new(load_ton_address(addr)?, abi, &url, callbacks);
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