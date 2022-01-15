pragma solidity =0.6.6;

import '@uniswap/v2-core/contracts/interfaces/IUniswapV2Callee.sol';
import '@uniswap/v2-core/contracts/interfaces/IUniswapV2Factory.sol';
import '@uniswap/v2-periphery/contracts/libraries/UniswapV2Library.sol';
import '@uniswap/v2-periphery/contracts/interfaces/IUniswapV2Router02.sol';
import '@uniswap/v2-periphery/contracts/interfaces/IERC20.sol';
import '@uniswap/v2-periphery/contracts/interfaces/IWETH.sol';

contract ExampleTriangleArbitrage is IUniswapV2Callee {
    address immutable factory;
    IUniswapV2Router02 immutable router02;

    address addr_bnb;
    address addr_cat;
    address addr_dog;
    address _pair_bnb_cat;
    address _pair_dog_cat;
    address _pair_dog_bnb;


    constructor(address _factory, address _router02,
        address _addr_bnb,
        address _addr_cat,
        address _addr_dog
    ) public {
        factory = _factory;
        router02 = IUniswapV2Router02(_router02);
        addr_bnb = _addr_bnb;
        addr_cat = _addr_cat;
        addr_dog = _addr_dog;

        _pair_bnb_cat = IUniswapV2Factory(_factory).getPair(addr_bnb, addr_cat);
        require(_pair_bnb_cat != address(0), "bnb cat pair not exist");
        _pair_dog_cat = IUniswapV2Factory(_factory).getPair(addr_dog, addr_cat);
        require(_pair_dog_cat != address(0), "dog cat pair not exist");
        _pair_dog_bnb = IUniswapV2Factory(_factory).getPair(addr_dog, addr_bnb);
        require(_pair_dog_bnb != address(0), "dog cat pair not exist");
    }


    event Log(string message, uint val);

    function thisaddress() public view returns (address) {
        return address(this);
    }


    receive() external payable {}

    function testTriangleFlash(address _me, uint _amount, address _amount_token) external {
        address[] memory path = new address[](4);

        path[0] = addr_cat;
        path[1] = addr_dog;
        path[2] = addr_bnb;
        path[3] = addr_cat;
        uint[] memory ins = UniswapV2Library.getAmountsIn(factory, _amount, path);

        bytes memory data = abi.encode(
            _me,
            ins,
            SwapType.GotCat
        );
        require(ins[3] == _amount, "ins[3] != _amount");
        IUniswapV2Pair(_pair_bnb_cat).swap(0, ins[3], address(this), data);
    }

    //           ins[3]
    //          _amount      ins[0]                  ins[1]               ins[2]
    // bnb/cat --(cat)-->I---(cat)---->dog/cat ------(dog)>----- dog/bnb --(bnb)->
    //                       outs[0]
    //      0       1      2      3
    //in:  cat -> dog -> bnb -> cat

    enum SwapType{GotCat, GotDog, GotBnb}

    function uniswapV2Call(address _sender, uint _amount0, uint _amount1, bytes calldata _data) external override {
        (address _me, uint[] memory ins, SwapType swap_type) = abi.decode(_data, (address, uint[], SwapType));
        require(_amount0 == 0, "gotcat: wbnb amount must be 0");
        require(_amount1 > 0, "gotcat: cat amount > 0");
        require(_amount1 == ins[3], "gotcat : received cat not equal to expect");

        IERC20(addr_cat).transfer(_pair_dog_cat, ins[0]);
        IUniswapV2Pair(_pair_dog_cat).swap(0, ins[1], _pair_dog_bnb, new bytes(0));
        IUniswapV2Pair(_pair_dog_bnb).swap(ins[2], 0, _pair_bnb_cat, new bytes(0));

        IERC20(addr_cat).transfer(_me, ins[3] - ins[0]);
    }
}
