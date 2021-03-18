pragma solidity >=0.6.0;
pragma experimental ABIEncoderV2;
pragma ignoreIntOverflow;
pragma AbiHeader expire;
pragma AbiHeader pubkey;
pragma AbiHeader time;

interface IAccept {
    function acceptTransfer(bytes payload) external payable;
}

/// @title Multisignature wallet
/// @author Tonlabs (https://tonlabs.io)
contract MultisigWallet is IAccept {

    /*
     *  Storage
     */

    struct Transaction {
        // Transaction Id.
        uint64 id;
        // Transaction confirmations from custodians.
        uint32 confirmationsMask;
        // Number of required confirmations.
        uint8 signsRequired;
        // Number of confirmations already received.
        uint8 signsReceived;
        // Public key of custodian queued transaction.
        uint256 creator;
        // Index of custodian.
        uint8 index;
        // Destination address of gram transfer.
        address payable dest;
        // Amount of nanograms to transfer.
        uint128 value;
        // Flags for sending internal message (see SENDRAWMSG in TVM spec).
        uint16 sendFlags;
        // Payload used as body of outbound internal message.
        TvmCell payload;
        // Bounce flag for header of outbound internal message.
        bool bounce;
    }

    /*
     *  Constants
     */
    uint8   constant MAX_QUEUED_REQUESTS = 5;
    uint64  constant EXPIRATION_TIME = 3600; // lifetime is 1 hour
    uint8   constant MAX_CUSTODIAN_COUNT = 32;
    uint128 constant MIN_VALUE = 1e6;
    uint    constant MAX_CLEANUP_TXNS = 40;

    // Send flags.
    // Forward fees for message will be paid from contract balance.
    uint8 constant FLAG_PAY_FWD_FEE_FROM_BALANCE = 1;
    // Tells node to ignore errors in action phase while outbound messages are sent.
    uint8 constant FLAG_IGNORE_ERRORS = 2;
    // Tells node to send all remaining balance.
    uint8 constant FLAG_SEND_ALL_REMAINING = 128;

    /*
     * Variables
     */

    // Public key of custodian who deployed a contract.
    uint256 m_ownerKey;
    // Binary mask with custodian requests (max 32 custodians).
    uint256 m_requestsMask;
    // Dictionary of queued transactions waiting confirmations.
    mapping(uint64 => Transaction) m_transactions;
    // Set of custodians, initiated in constructor, but values can be changed later in code.
    mapping(uint256 => uint8) m_custodians; // pub_key -> custodian_index
    // Read-only custodian count, initiated in constructor.
    uint8 m_custodianCount;
    // Default number of confirmations needed to execute transaction.
    uint8 m_defaultRequiredConfirmations;

    /*
    Exception codes:
    100 - message sender is not a custodian;
    102 - transaction does not exist;
    103 - operation is already confirmed by this custodian;
    107 - input value is too low;
    108 - wallet should have only one custodian;
    113 - Too many requests for one custodian;
    117 - invalid number of custodians;
    121 - payload size is too big;
    */

    /*
     *  Events
     */
    event TransferAccepted(bytes payload);

    /*
     * Runtime functions
     */
    function tvm_ctos(TvmCell cell) private pure returns (uint /* slice */) {}
    function tvm_tree_cell_size(uint slice) private pure returns (uint, uint) {}

    /*
     * Constructor
     */

    /// @dev Internal function called from constructor to initialize custodians.
    function _initialize(uint256[] owners, uint8 reqConfirms) inline private {
        uint8 ownerCount = 0;
        m_ownerKey = owners[0];

        uint256 len = owners.length;
        for (uint256 i = 0; (i < len && ownerCount < MAX_CUSTODIAN_COUNT); i++) {
            uint256 key = owners[i];
            if (!m_custodians.exists(key)) {
                m_custodians[key] = ownerCount++;
            }
        }
        m_defaultRequiredConfirmations = ownerCount <= reqConfirms ? ownerCount : reqConfirms;
        m_custodianCount = ownerCount;
    }

    /// @dev Contract constructor.
    /// @param owners Array of custodian keys.
    /// @param reqConfirms Default number of confirmations required for executing transaction.
    constructor(uint256[] owners, uint8 reqConfirms) public {
        require(msg.pubkey() == tvm.pubkey(), 100);
        require(owners.length > 0 && owners.length <= MAX_CUSTODIAN_COUNT, 117);
        tvm.accept();
        _initialize(owners, reqConfirms);
    }

    /*
     * Inline helper macros
     */

    /// @dev Returns queued transaction count by custodian with defined index.
    function _getMaskValue(uint256 mask, uint8 index) inline private pure returns (uint8) {
        return uint8((mask >> (8 * uint256(index))) & 0xFF);
    }

    /// @dev Increment queued transaction count by custodian with defined index.
    function _incMaskValue(uint256 mask, uint8 index) inline private pure returns (uint256) {
        return mask + (1 << (8 * uint256(index)));
    }

    /// @dev Decrement queued transaction count by custodian with defined index.
    function _decMaskValue(uint256 mask, uint8 index) inline private pure returns (uint256) {
        return mask - (1 << (8 * uint256(index)));
    }

    /// @dev Checks bit with defined index in the mask.
    function _checkBit(uint32 mask, uint8 index) inline private pure returns (bool) {
        return (mask & (uint32(1) << index)) != 0;
    }

    /// @dev Checks if object is confirmed by custodian.
    function _isConfirmed(uint32 mask, uint8 custodianIndex) inline private pure returns (bool) {
        return _checkBit(mask, custodianIndex);
    }

    /// @dev Sets custodian confirmation bit in the mask.
    function _setConfirmed(uint32 mask, uint8 custodianIndex) inline private pure returns (uint32) {
        mask |= (uint32(1) << custodianIndex);
        return mask;
    }

    /// @dev Checks that custodian with supplied public key exists in custodian set.
    function _findCustodian(uint256 senderKey) inline private view returns (uint8) {
        (bool exists, uint8 index) = m_custodians.fetch(senderKey);
        require(exists, 100);
        return index;
    }

    /// @dev Generates new id for object.
    function _generateId() inline private view returns (uint64) {
        return (uint64(now) << 32) | (tvm.transLT() & 0xFFFFFFFF);
    }

    /// @dev Returns timestamp after which transactions are treated as expired.
    function _getExpirationBound() inline private view returns (uint64) {
        return (uint64(now) - EXPIRATION_TIME) << 32;
    }

    /// @dev Returns transfer flags according to input value and `allBalance` flag.
    function _getSendFlags(uint128 value, bool allBalance) inline private pure returns (uint8, uint128) {        
        uint8 flags = FLAG_IGNORE_ERRORS | FLAG_PAY_FWD_FEE_FROM_BALANCE;
        if (allBalance) {
            flags = FLAG_IGNORE_ERRORS | FLAG_SEND_ALL_REMAINING;
            value = uint128(address(this).balance);
        }
        return (flags, value);
    }

    /*
     * Public functions
     */

    /// @dev A payable method for accepting incoming funds. Generates
    /// an event with incoming payload.
    /// @param payload Payload from message body.
    function acceptTransfer(bytes payload) external override payable {
        emit TransferAccepted(payload);
    }

    /// @dev Allows custodian if she is the only owner of multisig to transfer funds with minimal fees.
    /// @param dest Transfer target address.
    /// @param value Amount of funds to transfer.
    /// @param bounce Bounce flag. Set true if need to transfer funds to existing account;
    /// set false to create new account.
    /// @param flags `sendmsg` flags.
    /// @param payload Tree of cells used as body of outbound internal message.
    function sendTransaction(
        address payable dest,
        uint128 value,
        bool bounce,
        uint8 flags,
        TvmCell payload) public view
    {
        require(m_custodianCount == 1, 108);
        require(msg.pubkey() == m_ownerKey, 100);
        tvm.accept();
        dest.transfer(value, bounce, flags, payload);
    }

    /// @dev Allows custodian to submit and confirm new transaction.
    /// @param dest Transfer target address.
    /// @param value Nanograms value to transfer.
    /// @param bounce Bounce flag. Set true if need to transfer grams to existing account; set false to create new account.
    /// @param allBalance Set true if need to transfer all remaining balance.
    /// @param payload Tree of cells used as body of outbound internal message.
    /// @return transId Transaction ID.
    function submitTransaction(
        address payable dest,
        uint128 value,
        bool bounce,
        bool allBalance,
        TvmCell payload)
    public returns (uint64 transId)
    {
        uint256 senderKey = msg.pubkey();
        uint8 index = _findCustodian(senderKey);
        require(value >= MIN_VALUE, 107);
        (uint bits, uint cells) = tvm_tree_cell_size(tvm_ctos(payload));
        require(bits < 8192 && cells < 8, 121);
        _removeExpiredTransactions();
        require(_getMaskValue(m_requestsMask, index) < MAX_QUEUED_REQUESTS, 113);
        tvm.accept();

        (uint8 flags, uint128 realValue) = _getSendFlags(value, allBalance);        
        uint8 requiredSigns = m_defaultRequiredConfirmations;

        if (requiredSigns == 1) {
            dest.transfer(realValue, bounce, flags, payload);
            return 0;
        } else {
            m_requestsMask = _incMaskValue(m_requestsMask, index);
            uint64 trId = _generateId();
            Transaction txn = Transaction(trId, 0/*mask*/, requiredSigns, 0/*signsReceived*/,
                senderKey, index, dest, realValue, flags, payload, bounce);

            _confirmTransaction(trId, txn, index);
            return trId;
        }
    }

    /// @dev Allows custodian to confirm a transaction.
    /// @param transactionId Transaction ID.
    function confirmTransaction(uint64 transactionId) public {
        uint8 index = _findCustodian(msg.pubkey());
        _removeExpiredTransactions();
        (bool trexists, Transaction  txn) = m_transactions.fetch(transactionId);        
        require(trexists, 102);
        require(!_isConfirmed(txn.confirmationsMask, index), 103);
        tvm.accept();

        _confirmTransaction(transactionId, txn, index);
    }

    /*
     * Internal functions
     */

    /// @dev Confirms transaction by custodian with defined index.
    /// @param transactionId Transaction id to confirm.
    /// @param txn Transaction object to confirm.
    /// @param custodianIndex Index of custodian.
    function _confirmTransaction(uint64 transactionId, Transaction txn, uint8 custodianIndex) inline private {
        if ((txn.signsReceived + 1) >= txn.signsRequired) {
            txn.dest.transfer(txn.value, txn.bounce, txn.sendFlags, txn.payload);
            m_requestsMask = _decMaskValue(m_requestsMask, txn.index);
            delete m_transactions[transactionId];
        } else {
            txn.confirmationsMask = _setConfirmed(txn.confirmationsMask, custodianIndex);
            txn.signsReceived++;
            m_transactions[transactionId] = txn;
        }
    }

    /// @dev Removes expired transactions from storage.
    function _removeExpiredTransactions() inline private {
        uint64 marker = _getExpirationBound();
        (uint64 trId, Transaction txn, bool success) = m_transactions.min();

        bool needCleanup = success && (trId <= marker);
        if (!needCleanup) { return; }

        tvm.accept();
        uint i = 0;
        while (needCleanup && i < MAX_CLEANUP_TXNS) {
            // transaction is expired, remove it
            i++;
            m_requestsMask = _decMaskValue(m_requestsMask, txn.index);
            delete m_transactions[trId];

            (trId, txn, success) = m_transactions.next(trId);
            needCleanup = success && (trId <= marker);
        }        
        tvm.commit();
    }

    /*
     * Get methods
     */
    
    /// @dev Helper get-method for checking if custodian confirmation bit is set.
    /// @return confirmed True if confirmation bit is set.
    function isConfirmed(uint32 mask, uint8 index) public pure returns (bool confirmed) {
        confirmed = _isConfirmed(mask, index);
    }

    /// @dev Get-method that returns wallet configuration parameters.
    /// @return maxQueuedTransactions The maximum number of unconfirmed transactions that a custodian can submit.
    /// @return maxCustodianCount The maximum allowed number of wallet custodians.
    /// @return expirationTime Transaction lifetime in seconds.
    /// @return minValue The minimum value allowed to transfer in one transaction.
    /// @return requiredTxnConfirms The minimum number of confirmations required to execute transaction.
    function getParameters() public view
        returns (uint8 maxQueuedTransactions,
                uint8 maxCustodianCount,
                uint64 expirationTime,
                uint128 minValue,
                uint8 requiredTxnConfirms) {

        maxQueuedTransactions = MAX_QUEUED_REQUESTS;
        maxCustodianCount = MAX_CUSTODIAN_COUNT;
        expirationTime = EXPIRATION_TIME;
        minValue = MIN_VALUE;
        requiredTxnConfirms = m_defaultRequiredConfirmations;
    }

    /// @dev Get-method that returns transaction info by id.
    /// @return trans Transaction structure.
    /// Throws exception if transaction does not exist.
    function getTransaction(uint64 transactionId) public view
        returns (Transaction trans) {
        (bool exists, Transaction txn) = m_transactions.fetch(transactionId);
        require(exists, 102);
        trans = txn;
    }

    /// @dev Get-method that returns array of pending transactions.
    /// Returns not expired transactions only.
    /// @return transactions Array of queued transactions.
    function getTransactions() public view returns (Transaction[] transactions) {
        uint64 bound = _getExpirationBound();
        (uint64 id, Transaction txn, bool success) = m_transactions.min();
        while (success) {
            // returns only not expired transactions
            if (id > bound) {
                transactions.push(txn);
            }
            (id, txn, success) = m_transactions.next(id);
        }
    }

    /// @dev Get-method that returns submitted transaction ids.
    /// @return ids Array of transaction ids.
    function getTransactionIds() public view returns (uint64[] ids) {
        uint64 trId = 0;
        bool success = false;
        (trId, , success) = m_transactions.min();
        while (success) {
            ids.push(trId);
            (trId, , success) = m_transactions.next(trId);
        }
    }

    /// @dev Helper structure to return information about custodian.
    /// Used in getCustodians().
    struct CustodianInfo {
        uint8 index;
        uint256 pubkey;
    }

    /// @dev Get-method that returns info about wallet custodians.
    /// @return custodians Array of custodians.
    function getCustodians() public view returns (CustodianInfo[] custodians) {
        (uint256 key, uint8 index, bool success) = m_custodians.min();
        while (success) {
            custodians.push(CustodianInfo(index, key));
            (key, index, success) = m_custodians.next(key);
        }
    }    

    /*
     * Fallback and receive functions to receive simple transfers.
     */
    
    fallback () external payable {}

    receive () external payable {}
}