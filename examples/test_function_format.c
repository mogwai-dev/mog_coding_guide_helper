/*****************************/
/* Author: Test              */
/* Date: 2025-12-10          */
/* Purpose: Test function    */
/*****************************/

// 正しいフォーマット
static void
foobar(int a)
{
    return;
}

// 間違い1: 関数名と括弧が同じ行、括弧も同じ行
static void wrong_format(int a) {
    return;
}

// 間違い2: 戻り値と関数名が同じ行
static void wrong_format2(int a)
{
    return;
}
