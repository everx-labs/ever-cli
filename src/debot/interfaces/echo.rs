use serde_json::Value;

pub const ECHO_ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "echo",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"request","type":"bytes"}
			],
			"outputs": [
				{"name":"response","type":"bytes"}
			]
		},
		{
			"name": "constructor",
			"inputs": [
			],
			"outputs": [
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

pub struct Echo {}
impl Echo {
    fn echo(answer_id: u32, request: &str) -> (u32, Value) {
        ( answer_id, json!({ "response": hex::encode(request.as_bytes()) }) )
    }

    pub fn call(func: &str, args: &Value) -> (u32, Value) {
        match func {
            "echo" => {
                let answer_id = u32::from_str_radix(args["answerId"].as_str().unwrap(), 10).unwrap();
                let request_vec = hex::decode(args["request"].as_str().unwrap()).unwrap();
                let request = std::str::from_utf8(&request_vec).unwrap();
                Self::echo(answer_id, request)
            },
            _ => panic!("interface function not found"),
        }
    }
}
