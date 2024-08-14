pragma ton-solidity >=0.47.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
import "https://raw.githubusercontent.com/everx-labs/debots/main/Debot.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/ConfirmInput/ConfirmInput.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Menu/Menu.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/AddressInput/AddressInput.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/AmountInput/AmountInput.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/UserInfo/UserInfo.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/SigningBoxInput/SigningBoxInput.sol";
import "ICompleted.sol";

contract Invoked is Debot {
    
    bytes m_icon;
    address m_invoker;
    uint64 m_arg1;
    string m_arg2;
    bool m_arg3;
    uint32 m_arg4;
    address m_arg5;
    uint256 m_arg6;
    mapping(uint32 => Data) m_arg7;

    function setIcon(bytes icon) public {
        require(msg.pubkey() == tvm.pubkey(), 100);
        tvm.accept();
        m_icon = icon;
    }

    /// @notice Returns Metadata about DeBot.
    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "InvokeTest";
        version = "0.1.0";
        publisher = "EverX";
        caption = "For testing invokes";
        author = "EverX";
        support = address(0);
        hello = "Hello";
        language = "en";
        dabi = m_debotAbi.get();
        icon = m_icon;
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [Terminal.ID, AddressInput.ID, AmountInput.ID, ConfirmInput.ID, Menu.ID, UserInfo.ID];
    }

    /// @notice Entry point function for DeBot.
    function start() public override {
        uint256[] empty;
        SigningBoxInput.get(tvm.functionId(setSignBoxHandle), "Enter keys", empty);
    }

    function setSignBoxHandle(uint32 handle) public {
        require(handle != 0);
        this.setDataOnchain{
            abiVer: 2, extMsg: true, sign: true,
            time: 0, expire: 0, pubkey: 0, signBoxHandle: handle,
            callbackId: tvm.functionId(onSuccess),
            onErrorId: tvm.functionId(onError)
        }();
    }

    function onSuccess() public {

    }

    function onError(uint32 sdkError, uint32 exitCode) public {
        sdkError; exitCode;
        revert();
    }

    function setDataOnchain() public {
        require(msg.pubkey() == tvm.pubkey());
        tvm.accept();
    }

    //
    // Invoke funcitons
    //

    function invokeTest(uint64 arg1, string arg2, bool arg3, uint32 arg4, address arg5, uint256 arg6, mapping(uint32 => Data) arg7) public {
        m_invoker = msg.sender;
        AmountInput.get(tvm.functionId(checkArg1), "Enter arg1 as amount:", 9, 0, 100 ton);
        Terminal.input(tvm.functionId(checkArg2), "Enter arg2 as string:", false);
        Terminal.print(0, "Print smthg");
        ConfirmInput.get(tvm.functionId(checkArg3), "Enter arg3 as boolean:");
        MenuItem[] items;
        for(uint32 i = 0; i < arg4 + 1; i++) {
            items.push(MenuItem(format("Item {}", i), "", tvm.functionId(checkArg4)) );
        }
        Menu.select("Enter arg4 as menu index:", "", items);
        AddressInput.get(tvm.functionId(checkArg5), "Enter arg5 as address:");
        UserInfo.getPublicKey(tvm.functionId(checkArg6));
        m_arg1 = arg1;
        m_arg2 = arg2;
        m_arg3 = arg3;
        m_arg4 = arg4;
        m_arg5 = arg5;
        m_arg6 = arg6;
        m_arg7 = arg7;
    }

    // ----------------------------------------------------

    function checkArg1(uint128 value) public {
        require(value == m_arg1, 201);
    }

    function checkArg2(string value) public {
        require(value == m_arg2, 202);
    }

    function checkArg3(bool value) public {
        require(value == m_arg3, 203);
    }

    function checkArg4(uint32 index) public {
        require(index == m_arg4, 204);
    }

    function checkArg5(address value) public {
        require(value == m_arg5, 205);
    }

    function checkArg6(uint256 value) public {
        require(value == m_arg6, 206);
        IOnInvokeCompleted(m_invoker).OnInvokeCompleted(Status.Passed, m_arg7);
    }
}