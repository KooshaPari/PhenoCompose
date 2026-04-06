//go:build ignore

// C-export layer for NVMS Core
// Build with: go build -buildmode=c-archive -o nvms_core.a .
// Then link with Rust via cgo

package main

/*
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>

typedef struct NvmsInstance NvmsInstance;

typedef enum {
    NVMS_TIER_WASM = 1,
    NVMS_TIER_GVISOR = 2,
    NVMS_TIER_FIRECRACKER = 3,
} NvmsTier;

typedef enum {
    NVMS_STATUS_STOPPED = 0,
    NVMS_STATUS_STARTING = 1,
    NVMS_STATUS_RUNNING = 2,
    NVMS_STATUS_STOPPING = 3,
    NVMS_STATUS_ERROR = 4,
} NvmsStatus;

struct NvmsInstance {
    uint64_t id;
    NvmsTier tier;
    NvmsStatus status;
    char* name;
};
*/
import "C"

import (
	"runtime"
	"sync"
	"unsafe"
)

//export nvms_version
func nvms_version() *C.char {
	return C.CString("1.0.0-unified")
}

//export nvms_init
func nvms_init() C.int {
	return 0
}

//export nvms_instance_create
func nvms_instance_create(tier C.int, name *C.char) *C.NvmsInstance {
	instanceID := atomicAddUint64(&instanceCounter, 1)
	goName := C.GoString(name)

	cinst := (*C.NvmsInstance)(C.malloc(C.sizeof_NvmsInstance))
	cinst.id = C.uint64_t(instanceID)
	cinst.tier = tier
	cinst.status = C.NVMS_STATUS_RUNNING
	cinst.name = C.CString(goName)

	return cinst
}

//export nvms_instance_destroy
func nvms_instance_destroy(inst *C.NvmsInstance) C.int {
	if inst == nil {
		return -1
	}
	if inst.name != nil {
		C.free(unsafe.Pointer(inst.name))
	}
	C.free(unsafe.Pointer(inst))
	return 0
}

//export nvms_instance_start
func nvms_instance_start(inst *C.NvmsInstance) C.int {
	if inst == nil {
		return -1
	}
	inst.status = C.NVMS_STATUS_RUNNING
	return 0
}

//export nvms_instance_stop
func nvms_instance_stop(inst *C.NvmsInstance) C.int {
	if inst == nil {
		return -1
	}
	inst.status = C.NVMS_STATUS_STOPPED
	return 0
}

//export nvms_instance_status
func nvms_instance_status(inst *C.NvmsInstance) C.NvmsStatus {
	if inst == nil {
		return C.NVMS_STATUS_ERROR
	}
	return inst.status
}

var (
	instanceCounter uint64
	_              = sync.RWMutex{} // Placeholder for mutex
)

func atomicAddUint64(v *uint64, delta uint64) uint64 {
	*v += delta
	return *v
}

func main() {
	// Placeholder - not executed when built as c-archive
	_ = runtime.GOMAXPROCS(0)
}
