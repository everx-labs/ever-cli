pragma ton-solidity >=0.45.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/UserInfo/UserInfo.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "Sdk.sol";
import "Debot.sol";

contract Sample2 is Debot {

    address m_wallet;
    uint256 m_key;

    function setParams(address wallet, uint256 key) public {
        tvm.accept();
        m_wallet = wallet;
        m_key = key;
    }

    function start() public override {
        UserInfo.getAccount(tvm.functionId(setWallet));
        UserInfo.getPublicKey(tvm.functionId(setKey));
    }

    function setWallet(address value) public {
        require(value == m_wallet, 101);
        require(value != address(0), 104);
        Terminal.print(0, "Account is valid");
    }

    function setKey(uint256 value) public {
        require(value == m_key, 102);
        require(value != 0, 103);
        Terminal.print(0, "Public key is valid");
    }

    /// @notice Returns Metadata about DeBot.
    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "UserInfo";
        version = "0.1.0";
        publisher = "EverX";
        caption = "Test for UserInfo.";
        author = "EverX";
        support = address(0);
        hello = "Test DeBot.";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, UserInfo.ID ];
    }

}