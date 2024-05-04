#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct {
  const uint8_t *ptr;
  uintptr_t len;
} StrSlice;

typedef struct {
  int32_t a;
} Object;

typedef struct {
  int32_t a;
} List;

typedef struct {
  int32_t a;
} FunctionCall;

typedef enum {
  Int,
  Long,
  Float,
  Double,
  Bool,
  String,
  Object,
  List,
  FunctionCall,
  Symbol,
} Argument_Tag;

typedef struct {
  Argument_Tag tag;
  union {
    struct {
      int32_t int_;
    };
    struct {
      int64_t long_;
    };
    struct {
      float float_;
    };
    struct {
      double double_;
    };
    struct {
      bool bool_;
    };
    struct {
      StrSlice string;
    };
    struct {
      Object object;
    };
    struct {
      List list;
    };
    struct {
      FunctionCall function_call;
    };
    struct {
      StrSlice symbol;
    };
  };
} Argument;

typedef enum {
  None,
  Single,
  Many,
} ArgumentDefinition_Tag;

typedef struct {

} None_Body;

typedef struct {
  ArgumentDefinition_Tag tag;
  union {
    None_Body none;
    struct {
      Object single;
    };
    struct {
      List many;
    };
  };
} ArgumentDefinition;

void echo(Argument argument, ArgumentDefinition argument_definition);
