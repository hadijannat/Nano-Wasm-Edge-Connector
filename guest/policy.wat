(module
  ;; Import log function from host
  (import "host" "log" (func $log (param i32 i32)))
  
  ;; Define memory with 1 page (64KB)
  (memory (export "memory") 1)
  
  ;; Message data segments
  (data (i32.const 0) "Access GRANTED: admin role detected")
  (data (i32.const 64) "Access GRANTED: operator role")
  (data (i32.const 128) "Access GRANTED: viewer read-only access")
  (data (i32.const 192) "Access GRANTED: default policy")
  (data (i32.const 256) "Access DENIED: blocked flag present")
  (data (i32.const 320) "Access DENIED: operator cannot access sensitive")
  (data (i32.const 384) "Access DENIED: viewer cannot write")
  
  ;; Pattern data - stored at offset 512
  ;; "blocked":true = 14 bytes at 512
  ;; "admin" = 7 bytes at 544
  ;; "operator" = 10 bytes at 560
  ;; "viewer" = 8 bytes at 576
  ;; "secret" = 8 bytes at 592
  ;; "write" = 7 bytes at 608
  (data (i32.const 512) "\"blocked\":true")
  (data (i32.const 544) "\"admin\"")
  (data (i32.const 560) "\"operator\"")
  (data (i32.const 576) "\"viewer\"")
  (data (i32.const 592) "\"secret\"")
  (data (i32.const 608) "\"write\"")
  
  ;; Get input buffer pointer
  (func (export "get_input_buffer") (result i32)
    (i32.const 1024)
  )
  
  ;; Pattern matching - check if needle exists in haystack
  (func $contains (param $hay_ptr i32) (param $hay_len i32) (param $needle_ptr i32) (param $needle_len i32) (result i32)
    (local $i i32)
    (local $j i32)
    (local $found i32)
    
    (if (i32.gt_s (local.get $needle_len) (local.get $hay_len))
      (then (return (i32.const 0)))
    )
    
    (local.set $i (i32.const 0))
    
    (block $done
      (loop $outer
        (br_if $done (i32.gt_s (local.get $i) (i32.sub (local.get $hay_len) (local.get $needle_len))))
        
        (local.set $found (i32.const 1))
        (local.set $j (i32.const 0))
        
        (block $inner_done
          (loop $inner
            (br_if $inner_done (i32.ge_s (local.get $j) (local.get $needle_len)))
            
            (if (i32.ne 
                  (i32.load8_u (i32.add (local.get $hay_ptr) (i32.add (local.get $i) (local.get $j))))
                  (i32.load8_u (i32.add (local.get $needle_ptr) (local.get $j))))
              (then
                (local.set $found (i32.const 0))
                (br $inner_done)
              )
            )
            
            (local.set $j (i32.add (local.get $j) (i32.const 1)))
            (br $inner)
          )
        )
        
        (if (local.get $found)
          (then (return (i32.const 1)))
        )
        
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $outer)
      )
    )
    
    (i32.const 0)
  )
  
  ;; Main policy evaluation
  (func (export "evaluate_access") (param $ptr i32) (param $len i32) (result i32)
    ;; Rule 1: Check "blocked":true
    (if (call $contains (local.get $ptr) (local.get $len) (i32.const 512) (i32.const 14))
      (then
        (call $log (i32.const 256) (i32.const 35))
        (return (i32.const 0))
      )
    )
    
    ;; Rule 2: Check "admin"
    (if (call $contains (local.get $ptr) (local.get $len) (i32.const 544) (i32.const 7))
      (then
        (call $log (i32.const 0) (i32.const 35))
        (return (i32.const 1))
      )
    )
    
    ;; Rule 3: Check "operator"
    (if (call $contains (local.get $ptr) (local.get $len) (i32.const 560) (i32.const 10))
      (then
        ;; Check for "secret" - deny if present
        (if (call $contains (local.get $ptr) (local.get $len) (i32.const 592) (i32.const 8))
          (then
            (call $log (i32.const 320) (i32.const 47))
            (return (i32.const 0))
          )
        )
        (call $log (i32.const 64) (i32.const 29))
        (return (i32.const 1))
      )
    )
    
    ;; Rule 4: Check "viewer"
    (if (call $contains (local.get $ptr) (local.get $len) (i32.const 576) (i32.const 8))
      (then
        ;; Check for "write" - deny if present
        (if (call $contains (local.get $ptr) (local.get $len) (i32.const 608) (i32.const 7))
          (then
            (call $log (i32.const 384) (i32.const 34))
            (return (i32.const 0))
          )
        )
        (call $log (i32.const 128) (i32.const 39))
        (return (i32.const 1))
      )
    )
    
    ;; Default: allow
    (call $log (i32.const 192) (i32.const 30))
    (i32.const 1)
  )
)
