#ifndef coxy_vm_h
#define coxy_vm_h

#include "chunk.h"
#include "object.h"
#include "table.h"
#include "value.h"
#include <_types/_uint8_t.h>
#include <stddef.h>

#define FRAMES_MAX 64
#define STACK_MAX (FRAMES_MAX * UINT8_COUNT)

typedef struct {
  ObjClosure *closure;
  uint8_t *ip;
  Value *slots;
} CallFrame;

typedef struct {
  CallFrame frames[FRAMES_MAX];
  int frameCount;
  Value stack[STACK_MAX];
  Value *stackTop;
  Table globals;
  Table strings;
  ObjString *initString;
  ObjUpvalue *openUpvalues;
  Obj *objects;
  int grayCount;
  int grayCapacity;
  Obj **grayStack;
  size_t bytesAllocated;
  size_t nextGC;
} VM;

typedef enum {
  INTERPRET_OK,
  INTERPRET_COMPILER_ERROR,
  INTERPRET_RUNTIME_ERROR,
} InterpretResult;

extern VM vm;

void initVM();
void freeVM();
InterpretResult interpret(const char *source);
void push(Value value);
Value pop();
#endif
