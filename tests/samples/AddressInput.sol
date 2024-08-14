pragma ton-solidity >=0.35.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "Debot.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/AddressInput/AddressInput.sol";

contract AddressInputDebot is Debot {
    address m_addr = address(0xea5be6a13f20fcdfddc2c2b0d317dfeab56718249b090767e5940137b7af89f1);
    function start() public override {
        AddressInput.get(tvm.functionId(setAddress), "Enter Address:");
    }

    function setAddress(address value) public {
        if (m_addr == value) {
            Terminal.print(0, "AddressInput tests completed!");
        }
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, AddressInput.ID ];
    }

    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "AddressInput example DeBot";
        version = "0.1.0";
        publisher = "EverX";
        caption = "How to use the AddressInput interface";
        author = "EverX";
        support = address(0);
        hello = "Hello, i am an AddressInput example DeBot.";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }
}