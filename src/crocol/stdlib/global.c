#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

/**
 * Representation of a croco str
 * https://mapping-high-level-constructs-to-llvm-ir.readthedocs.io/en/latest/appendix-a-how-to-implement-a-string-type-in-llvm/
 * {
 *   ptr: i8*,
 *   len: isize,
 *   max_len: isize
 * }
 * 
 * this uses a different size depending on the host's architecture, for performance reasons
*/
typedef struct
{
  char *ptr;
  size_t len;
  size_t max_len;
} CrocoStr;

/**
 * Representation of a croco array
 */
typedef struct
{
  void *ptr;
  size_t len;
  size_t max_len;
} CrocoArray;

/**
 * Throws a CrocoError at runtime
 */
void _croco_error(char *file, uint32_t line, char *message, char *hint)
{
  fprintf(stderr, "Runtime error: %s\n", message);

  if (hint)
  {
    fprintf(stderr, "Hint: %s\n", hint);
  }

  if (file)
  {
    fprintf(stderr, "\nIn file %s:%lu\n", file, (unsigned long)line + 1);
  }

  exit(1);
}

/**
 * Resizes a CrocoStr if needed
 */
void _croco_str_resize(CrocoStr *string, size_t new_len)
{
  if (new_len <= string->max_len)
  {
    return;
  }

  string->ptr = (char *)realloc(string->ptr, new_len);
  string->max_len = new_len;
}

/**
 * Compares two CrocoStr
 * Returns 0 if both strings are equal
 * Returns < 0 if the first string is inferior to the second
 */
char _croco_str_cmp(CrocoStr *string1, CrocoStr *string2)
{
  size_t pos = 0;

  // compare each character of the strings until we find a difference, or until
  // one of the string is finished
  while (string1->len > pos && string2->len > pos)
  {
    char diff = string1->ptr[pos] - string2->ptr[pos];

    if (diff != 0)
    {
      return diff;
    }

    pos++;
  }

  // same characters and same length, same strings
  if (string1->len == string2->len)
  {
    return 0;
  }
  // the first string is shorter therefore inferior
  else if (string1->len == pos)
  {
    return -1;
  }
  // the second string is inferior
  else
  {
    return 1;
  }
}

/**
 * Casts a `str` into a `num`
 */
float _as_str_num(CrocoStr string)
{
  float res;

  // the number should be less than 100 digits, right ?!?!
  char tmp_str[100];
  sprintf(tmp_str, "%.*s", (int)string.len, string.ptr);

  int success = sscanf(tmp_str, "%f", &res);

  if (!success)
  {
    _croco_error(NULL, 0, "cannot cast a str to num", NULL);
  }

  return res;
}

/**
 * Casts a `fnum` into a `str`
 */
void _as_fnum_str(float fnum, CrocoStr *string_res)
{
  _croco_str_resize(string_res, 100);

  // this is going to null terminate our string but this doesn't really matter.
  sprintf(string_res->ptr, "%g", fnum);
  string_res->len = strlen(string_res->ptr);
}

/**
 * Casts a `num` into a `str`
 */
void _as_num_str(int32_t num, CrocoStr *string_res)
{
  // 10 chars is the max int size, 1 char for the sign, 1 char for the null byte
  _croco_str_resize(string_res, 12);
  sprintf(string_res->ptr, "%d", num);
  string_res->len = strlen(string_res->ptr);
}

/**
 * Exits if `assertion` is false
 */
void assert(bool assertion)
{
  if (!assertion)
  {
    fprintf(stderr, "Assertion failed!");
    exit(1);
  }
}

/**
 * Prints to stderr
 */
void eprint(CrocoStr string)
{
  fprintf(stderr, "%.*s", (int)string.len, string.ptr);
}

/**
 * Prints to stderr with a line feed
 */
void eprintln(CrocoStr string)
{
  fprintf(stderr, "%.*s\n", (int)string.len, string.ptr);
}
/**
 * Prints to stdout
 */
void print(CrocoStr string)
{
  printf("%.*s", (int)string.len, string.ptr);
}

/**
 * Prints to stdout with a line feed
 */
void println(CrocoStr string)
{
  printf("%.*s\n", (int)string.len, string.ptr);
}