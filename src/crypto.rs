use ton_client::InteropContext;
use ton_client::{tc_json_request, InteropString};
use ton_client::{tc_read_json_response, tc_destroy_json_response, JsonResponse};
use serde_json::{Value};
use ton_client::{tc_create_context, tc_destroy_context};

const HD_PATH: &str = "m/44'/396'/0'/0/0";
const WORD_COUNT: u8 = 12;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
pub struct KeyPair {
    pub public: String,
    pub secret: String,
}

fn interop_string_from(s: &String) -> InteropString {
    InteropString {
        content: s.as_ptr(),
        len: s.len() as u32,
    }
}

fn interop_string_to_string(istr: InteropString) -> String {
    unsafe {
        let utf8 = std::slice::from_raw_parts(istr.content, istr.len as usize);
        String::from_utf8(utf8.to_vec()).unwrap()
    }
}

// TODO: SdkClient structure is a temporary solution to use crypto
// functions from sdk. Remove it when ton-client-rs will support all
// necessary functions.
struct SdkClient {
    context: InteropContext,
}

impl SdkClient {
    fn new() -> Self {
        let context: InteropContext;
        unsafe {
            context = tc_create_context()
        }
        Self { context }
    }

    

    fn request(
        &self,
        method_name: &str,
        params: Value,
    ) -> Result<String, String> {
        unsafe {
            let params_json = if params.is_null() { String::new() } else { params.to_string() };
            let response_ptr = tc_json_request(
                self.context,
                interop_string_from(&method_name.to_string()),
                interop_string_from(&params_json),
            );
            let interop_response = tc_read_json_response(response_ptr);
            let response = JsonResponse {
                result_json: interop_string_to_string(interop_response.result_json),
                error_json: interop_string_to_string(interop_response.error_json),
            };
             //interop_response.to_response();
            tc_destroy_json_response(response_ptr);
            if response.error_json.is_empty() {
                Ok(response.result_json)
            } else {
                Err(response.error_json)
            }
        }
    }
}

impl Drop for SdkClient {
    fn drop(&mut self) {
        unsafe {
            tc_destroy_context(self.context)
        }
    }
}

fn parse_string(r: String) -> Result<String, String> {
    let json = serde_json::from_str(&r).map_err(|e| format!("failed to parse sdk client result: {}", e))?;
    if let Value::String(s) = json {
        Ok(s)
    } else {
        Err("failed to parse sdk client result: string expected".to_string())
    }
}

fn gen_seed_phrase() -> Result<String, String> {
    let client = SdkClient::new();
    parse_string(client.request(
        "crypto.mnemonic.from.random",
        json!({
            "dictionary": 1,
            "wordCount": WORD_COUNT
        })
    )?)
}

pub fn generate_keypair_from_mnemonic(mnemonic: &str) -> Result<KeyPair, String> {
    let client = SdkClient::new();

    let hdk_master = parse_string(client.request(
        "crypto.hdkey.xprv.from.mnemonic",
        json!({
            "dictionary":1,
            "wordCount": WORD_COUNT,
            "phrase": mnemonic.to_string(),
        })
    )?)?;

    let hdk_root = parse_string(client.request(
        "crypto.hdkey.xprv.derive.path",
        json!({
            "serialized": hdk_master,
            "path": HD_PATH.to_string(),
            "compliant": false,
        })
    )?)?;

    let secret = parse_string(client.request(
        "crypto.hdkey.xprv.secret",
        json!({
            "serialized": hdk_root
        })
    )?)?;

    let mut keypair: KeyPair = serde_json::from_str(&client.request(
        "crypto.nacl.sign.keypair.fromSecretKey",
        json!(secret)
    )?)
    .map_err(|e| format!("failed to parse KeyPair from json: {}", e))?;

    // special case if secret contains public key too.
    let secret = hex::decode(&keypair.secret).unwrap();
    if secret.len() > 32 {
        keypair.secret = hex::encode(&secret[..32]);
    }
    Ok(keypair)
}

pub fn generate_mnemonic() -> Result<(), String> {
    let mnemonic = gen_seed_phrase()?;
    println!("Succeeded.");
    println!(r#"Seed phrase: "{}""#, mnemonic);
    Ok(())
}

pub fn extract_pubkey(mnemonic: &str) -> Result<(), String> {
    let keypair = generate_keypair_from_mnemonic(mnemonic)?;
    println!("Succeeded.");
    println!("Public key: {}", keypair.public);
    println!();
    qr2term::print_qr(&keypair.public).unwrap();
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let mnemonic = "multiply extra monitor fog rocket defy attack right night jaguar hollow enlist";
        let keypair = generate_keypair_from_mnemonic(mnemonic).unwrap();
        assert_eq!(&keypair.public, "757221fe3d4992e44632e75e700aaf205d799cb7373ee929273daf26adf29e56");
        assert_eq!(&keypair.secret, "30e3bc5e67af2b0a72971bcc11256e83d052c6cb861a69a19a8af88922fadf3a");

        let mnemonic = "penalty nut enrich input palace flame safe session torch depth various hunt";
        let keypair = generate_keypair_from_mnemonic(mnemonic).unwrap();
        assert_eq!(&keypair.public, "8cf557aab2666867a1174e3147d89ddf28c2041a7322522276cd1cf1df47ae73");
        assert_eq!(&keypair.secret, "f63d3d11e0dc91f730f22d5397f269e01f1a5f984879c8581ac87f099bfd3b3a");
    }

}
