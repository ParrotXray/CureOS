#ifndef __STDBOOL_H
#define __STDBOOL_H
 
#ifndef __cplusplus
 
/* 如果編譯器不支援 _Bool 類型，則將其定義為 unsigned char */
#ifndef _Bool
#define _Bool unsigned char
#endif

/* 定義 bool 為 _Bool 的別名 */
#define bool _Bool

/* 定義 true 和 false 常數 */
#define true 1
#define false 0

/* 定義 __bool_true_false_are_defined 以表示已定義 true 和 false */
#define __bool_true_false_are_defined 1

#else /* __cplusplus */

/* 
* 對於 C++ 編譯器，這些定義已經存在，
* 我們只需定義 __bool_true_false_are_defined
*/
#define __bool_true_false_are_defined 1

#endif /* __cplusplus */

#endif /* __STDBOOL_H */