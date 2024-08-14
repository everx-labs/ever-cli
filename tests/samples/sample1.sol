pragma ton-solidity >=0.40.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/SigningBoxInput/SigningBoxInput.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "Sdk.sol";
import "Debot.sol";

contract ExampleContract is Debot {

    function start() public override {
        SigningBoxInput.get(tvm.functionId(setSigningBoxHandle), "Enter my signing keys:", [tvm.pubkey()]);
    }

    function setSigningBoxHandle(uint32 handle) public {
        Terminal.print(0, format("Signing Box Handle: {}", handle));
        uint256 hash = sha256("test sign string");
        Sdk.signHash(tvm.functionId(setSignature), handle, hash);
    }

    function setSignature(bytes signature) public {
        require(signature.length == 64, 200);
        uint256 hash = sha256("test sign string");
        require(tvm.checkSign(hash, signature.toSlice(), tvm.pubkey()), 201);
        Terminal.print(0,"test sign hash passed");
    }

    /// @notice Returns Metadata about DeBot.
    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "SigningBoxInput";
        version = "0.1.0";
        publisher = "EverX";
        caption = "Test for SigningBoxInput.";
        author = "EverX";
        support = address(0);
        hello = "Test DeBot.";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Terminal.ID, SigningBoxInput.ID ];
    }

}