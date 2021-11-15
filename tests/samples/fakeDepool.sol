pragma ton-solidity >=0.6.0;
pragma AbiHeader expire;
pragma AbiHeader time;

interface IFakeDePoolClient {
    function receiveAnswer(uint32 errcode, uint64 comment) external;
}

contract FakeDePool {

    uint static m_seed;
    uint64 m_stake;
    address m_sender;
    address m_receiver;
    uint32 m_withdrawal;
    uint32 m_total;
    bool m_reinvest;
    uint128 m_value;

    event StakeSigningRequested(uint32 electionId, address proxy);

    function sendAnswer() public pure {
        IFakeDePoolClient(msg.sender).receiveAnswer(11, 222);
    }

    function addOrdinaryStake(uint64 stake) public {
        m_stake = stake;
        m_sender = msg.sender;
        m_value = msg.value;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(11, 222);
    }

    function addVestingStake(uint64 stake, address beneficiary, uint32 withdrawalPeriod, uint32 totalPeriod) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = stake;
        m_receiver = beneficiary;
        m_withdrawal = withdrawalPeriod;
        m_total = totalPeriod;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(0, 0);
    }

    function addLockStake(uint64 stake, address beneficiary, uint32 withdrawalPeriod, uint32 totalPeriod) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = stake;
        m_receiver = beneficiary;
        m_withdrawal = withdrawalPeriod;
        m_total = totalPeriod;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(0, 0);
    }

    function withdrawFromPoolingRound(uint64 withdrawValue) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = withdrawValue;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(0, 0);
    }

    function withdrawPart(uint64 withdrawValue) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = withdrawValue;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(0, 0);
    }

    function withdrawAll() public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_reinvest = false;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(0, 0);
    }

    function cancelWithdrawal() public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_reinvest = true;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(0, 0);
    }

    function transferStake(address dest, uint64 amount) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = amount;
        m_receiver = dest;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(0, 0);
    }

    function ticktock() public {
        m_sender = msg.sender;
        m_value = msg.value;
        msg.sender.transfer({value: 123, flag: 1});

        emit StakeSigningRequested(1, address(2));
    }

    function receiveFunds() public {
        m_sender = msg.sender;
        m_value = msg.value;
    }

    function getData() public view returns (uint64 stake, address sender, address receiver,
        uint32 withdrawal, uint32 total, bool reinvest, uint128 value) {
        return (m_stake, m_sender, m_receiver, m_withdrawal, m_total, m_reinvest, m_value);
    }

    function setVestingDonor(address donor) public {
        m_receiver = donor;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(0, 0);
    }

    function setLockDonor(address donor) public {
        m_receiver = donor;
        IFakeDePoolClient(msg.sender).receiveAnswer{value: 123456789}(0, 0);
    }

    function error(uint code) public pure {
        revert(code);
    }

    function outOfGas() public pure {
        mapping(uint => uint) map;
        uint k = 0;
        while (k <= 9999) {
            map[k] = k;
        }
    }
}