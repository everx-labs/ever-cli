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
pub fn convert_token(amount: &str) -> Result<String, String> {
    let parts: Vec<&str> = amount.split(".").collect();
    if parts.len() >= 1 && parts.len() <= 2 {
        let mut result = String::new();
        result += parts[0];
        if parts.len() == 2 {
            let fraction = format!("{:0<9}", parts[1]);
            if fraction.len() != 9 {
                return Err("invalid fractional part".to_string());
            }
            result += &fraction;
        } else {
            result += "000000000";
        }
        u64::from_str_radix(&result, 10)
            .map_err(|e| format!("failed to parse amount: {}", e))?;
        
        return Ok(result);
    }
    Err("Invalid amout value".to_string())
}