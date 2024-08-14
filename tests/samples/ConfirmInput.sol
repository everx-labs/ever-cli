pragma ton-solidity >=0.35.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "Debot.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/ConfirmInput/ConfirmInput.sol";

contract ConfirmInputDebot is Debot {
    function start() public override {
       ConfirmInput.get(tvm.functionId(setResult), "Select:");
    }

    function setResult(bool value) public {
        if (value) {
            Terminal.print(0, "ConfirmInput tests completed!");
        }
    }
    
    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, ConfirmInput.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "ConfirmInput example DeBot";
        version = "0.1.0";
        publisher = "EverX";
        caption = "How to use the ConfirmInput interface";
        author = "EverX";
        support = address(0);
        hello = "Hello, i am an ConfirmInput example DeBot.";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }
}