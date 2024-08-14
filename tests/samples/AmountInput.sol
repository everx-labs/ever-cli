pragma ton-solidity >=0.35.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "Debot.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/AmountInput/AmountInput.sol";

contract AmountInputDebot is Debot {
    
    function start() public override {
        AmountInput.get(tvm.functionId(setAmountWithDecimals), "Enter amount of tons with decimals:",  9, 0, 100e9);
    }

    function setAmountWithDecimals(uint128 value) public {
        if (value == uint128(99456654321)) {   
            Terminal.print(0, "AmountInput tests completed!");
        }
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, AmountInput.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "AmountInput example DeBot";
        version = "0.1.0";
        publisher = "EverX";
        caption = "How to use the AmountInput interface";
        author = "EverX";
        support = address(0);
        hello = "Hello, i am an AmountInput example DeBot.";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }
}