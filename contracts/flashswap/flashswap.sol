pragma solidity =0.6.6;

import '../../../v2-core/contracts/interfaces/IUniswapV2Callee.sol';
import '../../../v2-core/contracts/interfaces/IUniswapV2Factory.sol';

import '../libraries/UniswapV2Library.sol';
import '../interfaces/IUniswapV2Router02.sol';
import '../interfaces/IERC20.sol';
import '../interfaces/IWETH.sol';

contract ExampleFlashSwap is IUniswapV2Callee {
    IUniswapV2Factory immutable factory;
    IUniswapV2Factory immutable factory_2;
    IUniswapV2Router02 immutable router02;
    IUniswapV2Router02 immutable router02_2;
    address immutable weth;

    constructor(address _factory, address _factory_2, address _router02, address _router02_2, address _weth) public {
        factory = IUniswapV2Factory(_factory);
        factory_2 = IUniswapV2Factory(_factory_2);
        router02 = IUniswapV2Router02(_router02);
        router02_2 = IUniswapV2Router02(_router02_2);
        weth = _weth;
    }

    function thisaddress() public view returns (address) {
        return address(this);
    }

    function testFlashSwap(address tokenx, address _tokenBorrow, uint _amount, address _me) external {
        address _pair_0 = factory.getPair(tokenx, _tokenBorrow);
        address _pair_2 = factory_2.getPair(tokenx, _tokenBorrow);
        address token0 = IUniswapV2Pair(_pair_0).token0();
        address token1 = IUniswapV2Pair(_pair_0).token1();


        address[]memory path = new address[](2);

        path[0] = token0 == _tokenBorrow ? token1 : token0;
        path[1] = _tokenBorrow;
        require(path[0] != path[1], "path[0] == path[1]");
        uint[] memory ins = router02.getAmountsIn(_amount, path);

        path[0] = _tokenBorrow;
        path[1] = token0 == _tokenBorrow ? token1 : token0;
        require(path[0] != path[1], "path[0] == path[1]");
        uint[] memory outs2 = router02_2.getAmountsOut(_amount, path);

        require(ins[1] == outs2[0], "ins[1] != outs2[0]");

        require(outs2[1] > ins[0], "profit not positive");


        address tokenBorrow = _tokenBorrow;
        bytes memory data = abi.encode(_me, outs2[1], ins[0], tokenBorrow, _pair_0, _pair_2, token0, token1);
        uint _amount0 = token0 == _tokenBorrow ? _amount : 0;
        uint _amount1 = token1 == _tokenBorrow ? _amount : 0;

        IUniswapV2Pair(_pair_0).swap(_amount0, _amount1, address(this), data);
    }

    receive() external payable {}

    function uniswapV2Call(address _sender, uint _amount0, uint _amount1, bytes calldata _data) external override {
        require(_amount0 > 0 || _amount1 > 0, "_amount0 or _amount1 must > 0");
        uint _amountToken = _amount0 == 0 ? _amount1 : _amount0;
        (address _me,uint allGet, uint needPay, address _tokenBorrow, address _pair_0, address pair_2, address token0,address token1)
        = abi.decode(_data, (address, uint, uint, address, address, address, address, address));

        address _tokenPay = IUniswapV2Pair(pair_2).token0() == _tokenBorrow ? IUniswapV2Pair(pair_2).token1() : IUniswapV2Pair(pair_2).token0();
        require(_tokenPay != _tokenBorrow, "tokenPay == tokenBorrow");

        address[] memory path = new address[](2);
        path[0] = _tokenBorrow;
        path[1] = _tokenPay;

        uint needIn = router02_2.getAmountsIn(needPay, path)[0];
        IERC20(_tokenBorrow).approve(address(router02_2), _amountToken);
        uint cost = router02_2.swapExactTokensForTokens(needIn, needPay, path, _pair_0, block.timestamp + 1 days)[0];

        require(cost == needIn, "cost != needIn");

        uint amountToken = _amountToken;
        IERC20(_tokenBorrow).transfer(_me, amountToken - cost);
    }
}
