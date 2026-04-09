{
    label start;

    let x = 10;
    let y = 20;

    let sum = x + y;
    print sum;

    let diff = y - x;
    print diff;

    let prod = x * y;
    print prod;

    let quo = y / x;
    print quo;

    let mod = y % x;
    print mod;

    jumpif greater x < y && y > 0;

    print "should not print";

    label greater;
    print "branch taken";

    call subroutine;

    label subroutine;
    print "in subroutine";
}