pragma ton-solidity >=0.35.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "Debot.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/NumberInput/NumberInput.sol";

contract NumberInputDebot is Debot {
    function start() public override {
        NumberInput.get(tvm.functionId(setNumber), "Enter number:", 0, 100);
    }

    function setNumber(int256 value) public {
        if (value == int256(79)) {   
            Terminal.print(0, "NumberInput tests completed!");
        }
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, NumberInput.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "NumberInput example DeBot";
        version = "0.1.0";
        publisher = "EverX";
        caption = "How to use the NumberInput interface";
        author = "EverX";
        support = address(0);
        hello = "Hello, i am an NumberInput example DeBot.";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }
}
