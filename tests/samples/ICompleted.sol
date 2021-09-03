pragma ton-solidity >=0.47.0;

enum Status {
    Passed,
    Failed
}

struct Data {
    bytes data;
}

interface IOnInvokeCompleted {
    function OnInvokeCompleted(Status status, mapping(uint32 => Data) ret1) external;
}

contract Test is IOnInvokeCompleted {
    function OnInvokeCompleted(Status status, mapping(uint32 => Data) ret1) external override {}
}