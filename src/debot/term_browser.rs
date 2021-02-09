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
use super::term_signing_box::TerminalSigningBox;
use crate::config::Config;
use crate::helpers::{create_client, load_ton_address, TonClient};
use std::io::{self, BufRead, Write};
use std::sync::{Arc, RwLock};
use ton_client::abi::{Abi, ParamsOfDecodeMessageBody, decode_message_body};
use ton_client::boc::{ParamsOfParse, parse_message};
use ton_client::crypto::SigningBoxHandle;
use ton_client::debot::{DEBOT_WC, BrowserCallbacks, DAction, DEngine, STATE_EXIT};
use std::collections::VecDeque;
use super::{SUPPORTED_INTERFACES, ECHO_ABI, Echo};

struct TerminalBrowser {
    state_id: u8,
    active_actions: Vec<DAction>,
    client: TonClient,
    msg_queue: VecDeque<String>,
}

impl TerminalBrowser {
    pub fn new(client: TonClient) -> Self {
        Self {
            state_id: 0,
            active_actions: vec![],
            client,
            msg_queue: Default::default(),
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

    async fn handle_interface_calls(&mut self, debot: &mut DEngine) -> Result<(), String> {
        for msg in self.msg_queue.drain(0..) {
            let parsed = parse_message(
                self.client.clone(),
                ParamsOfParse { boc: msg.clone() },
            ).map_err(|e| format!("{}", e))?;
            let body = parsed.parsed["body"].as_str().unwrap().to_owned();
            let iface_addr = parsed.parsed["dst"].as_str().unwrap();
            let wc_and_addr: Vec<_> = iface_addr.split(':').collect();
            let interface_id = wc_and_addr[1];
            let wc = i8::from_str_radix(wc_and_addr[0], 10).unwrap();
            if wc != DEBOT_WC {
                println!("invalid interface workchain id: {}", wc);
                continue;
            }
            if !SUPPORTED_INTERFACES.contains(&interface_id) {
                println!("DInterface {} not supported", interface_id);
                continue;
            }
            let decoded = decode_message_body(
                self.client.clone(),
                ParamsOfDecodeMessageBody {
                    abi: Abi::Json(ECHO_ABI.to_owned()),
                    body,
                    is_internal: true,
                },
            ).map_err(|e| format!(" failed to decode msg for interface: {}", e))?;
            debug!("call for interface id {}", interface_id);
            debug!("request: {} ({})", decoded.name, decoded.value.as_ref().unwrap());
            let (func_id, return_args) = Echo::call(&decoded.name, &decoded.value.unwrap());
            debug!("response: {} ({})", func_id, return_args);
            let result = debot.send(
                iface_addr.to_owned(), func_id, return_args.to_string()
            ).await;
            if let Err(e) = result {
                debug!("debot.send failed: {}", e);
                println!("debot call failed: {}", e);
            }
        }
        Ok(())
    }
}

struct Callbacks {
    browser: Arc<RwLock<TerminalBrowser>>,
}

impl Callbacks {
    pub fn new(browser: Arc<RwLock<TerminalBrowser>>) -> Self {
        Self { browser }
    }
}

#[async_trait::async_trait]
impl BrowserCallbacks for Callbacks {
    /// Debot asks browser to print message to user
    async fn log(&self, msg: String) {
        println!("{}", msg);
    }

    /// Debot is switched to another context.
    async fn switch(&self, ctx_id: u8) {
        let mut browser = self.browser.write().unwrap();
        debug!("switched to ctx {}", ctx_id);
        browser.state_id = ctx_id;
        if ctx_id == STATE_EXIT {
            println!("Debot shutdown");
            return;
        }

        browser.active_actions = vec![];
    }

    async fn switch_completed(&self) {
    }

    /// Debot asks browser to show user an action from the context
    async fn show_action(&self, act: DAction) {
        let mut browser = self.browser.write().unwrap();
        println!("{}) {}", browser.active_actions.len() + 1, act.desc);
        browser.active_actions.push(act);
    }

    // Debot engine asks user to enter argument for an action.
    async fn input(&self, prefix: &str, value: &mut String) {
        let stdio = io::stdin();
        let mut reader = stdio.lock();
        let mut writer = io::stdout();
        *value = input(prefix, &mut reader, &mut writer);
    }

    /// Debot engine requests keys to sign something
    async fn get_signing_box(&self) -> Result<SigningBoxHandle, String> {
        let terminal_box = TerminalSigningBox::new()?;
        let client = self.browser.read().unwrap().client.clone();
        let handle = ton_client::crypto::get_signing_box(
            client,
            terminal_box.keys,
        )
        .await
        .map(|r| r.handle)
        .map_err(|e| e.to_string())?;
        Ok(handle)
    }

    /// Debot asks to run action of another debot
    async fn invoke_debot(&self, debot: String, action: DAction) -> Result<(), String> {
        debug!("fetching debot {} action {}", &debot, action.name);
        println!("Invoking debot {}", &debot);
        let ton_cl = self.browser.read().unwrap().client.clone();
        let browser = Arc::new(
            RwLock::new(
                TerminalBrowser::new(ton_cl.clone())
            )
        );
        let callbacks = Arc::new(Callbacks::new(Arc::clone(&browser)));
        let mut debot_eng = DEngine::new_with_client(
            debot.clone(),
            None,
            ton_cl.clone(),
            callbacks,
        );
        debot_eng.fetch().await?;
        if let Err(e) = debot_eng.execute_action(&action).await {
            println!("Error. {}", e);
            return Ok(());
        }

        loop {
            let action = browser.read().unwrap().select_action();
            match action {
                Some(act) => {
                    if let Err(e) = debot_eng.execute_action(&act).await {
                        println!("Error. {}", e);
                        break;
                    }
                },
                None => break,
            }
        }
        Ok(())
    }

    async fn send(&self, message: String) {
        let mut browser = self.browser.write().unwrap();
        browser.msg_queue.push_back(message);
    }
}

pub(crate) fn input<R, W>(prefix: &str, reader: &mut R, writer: &mut W) -> String
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

pub async fn run_debot_browser(
    addr: &str,
    config: Config,
) -> Result<(), String> {
    println!("Connecting to {}", config.url);
    let ton = create_client(&config)?;
    let browser = Arc::new(RwLock::new(TerminalBrowser::new(ton.clone())));

    let callbacks = Arc::new(Callbacks::new(Arc::clone(&browser)));
    let mut debot = DEngine::new_with_client(load_ton_address(addr, &config)?, None, ton.clone(), callbacks);
    debot.start().await?;

    loop {
        browser.write().unwrap().handle_interface_calls(&mut debot).await?;
        let action = browser.read().unwrap().select_action();
        match action {
            Some(act) => debot.execute_action(&act).await?,
            None => break,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
