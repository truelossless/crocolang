#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <vcruntime.h>

/**
 Representation of a croco str
*/
typedef struct {
  char *ptr;
  size_t len;
  size_t max_len;
} CrocoStr;

/**
  Representation of a croco array
*/
typedef struct {
  void *ptr;
  size_t len;
  size_t max_len;
} CrocoArray;

/**
  Resizes a CrocoStr if needed
*/
void _croco_str_resize(CrocoStr *string, size_t new_len) {

  if (new_len <= string->max_len) {
    return;
  }

  realloc(string->ptr, new_len);
  string->max_len = new_len;
}

/**
  Casts a `str` into a `num`
*/
float _as_str_num(CrocoStr *string) {

  float res;

  // the number should be less than 100 digits, right ?!?!
  char tmp_str[100];
  sprintf(tmp_str, "%.*s", (int)string->len, string->ptr);

  int success = sscanf_s(tmp_str, "%f", &res);

  if (!success) {
    fprintf(stderr, "Runtime error: cannot convert %s to num", tmp_str);
    exit(1);
  }

  return res;
}

/**
  Casts a `num` into a `str`
*/
void _as_num_str(float num, CrocoStr *string_res) {
  _croco_str_resize(string_res, 100);

  // this is going to null terminate our string but this doesn't really matter.
  sprintf(string_res->ptr, "%g", num);
  string_res->len = strlen(string_res->ptr);
}

/**
  Exits if `assertion` is false
*/
void assert(bool assertion) {
  if (!assertion) {
    fprintf(stderr, "Assertion failed !");
    exit(1);
  }
}

/**
  Prints to stderr
*/
void eprint(CrocoStr *string) {
  fprintf(stderr, "%.*s", (int)string->len, string->ptr);
}

/**
  Prints to stderr with a line feed
*/
void eprintln(CrocoStr *string) {
  fprintf(stderr, "%.*s\n", (int)string->len, string->ptr);
}
/**
  Prints to stdout
*/
void print(CrocoStr *string) { printf("%.*s", (int)string->len, string->ptr); }

/**
  Prints to stdout with a line feed
*/
void println(CrocoStr *string) {
  printf("%.*s\n", (int)string->len, string->ptr);
}