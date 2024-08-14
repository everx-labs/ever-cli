pragma ton-solidity >=0.43.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;

import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "Debot.sol";

contract TerminalDebot is Debot {
    function start() public override {
        Terminal.input(tvm.functionId(setText), "Enter text", false);
    }

	function setText(string value) public {
        if (value == "Test value") {
            Terminal.print(0, "Terminal tests completed!");
        }
	}
    
    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID];
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "Terminal DeBot";
        version = "0.1.0";
        publisher = "EverX";
        caption = "How to use the Terminal interface";
        author = "EverX";
        support = address(0);
        hello = "Hello, i am a Terminal example DeBot.";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }
}
