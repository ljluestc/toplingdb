//! Java JNI bindings for ToplingDB

#[cfg(feature = "java-bindings")]
use jni::objects::{JByteArray, JClass, JObject, JString, JValue};
#[cfg(feature = "java-bindings")]
use jni::sys::{jbyteArray, jlong, jstring};
#[cfg(feature = "java-bindings")]
use jni::JNIEnv;
#[cfg(feature = "java-bindings")]
use std::ffi::CString;
#[cfg(feature = "java-bindings")]
use std::ptr;
#[cfg(feature = "java-bindings")]
use std::sync::Arc;

#[cfg(feature = "java-bindings")]
use crate::{DB, Options, ReadOptions, WriteOptions};

// Convert Rust DB pointer to Java long
#[cfg(feature = "java-bindings")]
fn db_to_handle(db: Arc<DB>) -> jlong {
    Box::into_raw(Box::new(db)) as jlong
}

// Convert Java long to Rust DB pointer
#[cfg(feature = "java-bindings")]
unsafe fn handle_to_db(handle: jlong) -> &'static Arc<DB> {
    &*(handle as *mut Arc<DB>)
}

// Release DB handle
#[cfg(feature = "java-bindings")]
unsafe fn release_db_handle(handle: jlong) {
    let _ = Box::from_raw(handle as *mut Arc<DB>);
}

#[cfg(feature = "java-bindings")]
#[no_mangle]
pub extern "system" fn Java_org_rocksdb_ToplingDB_open(
    env: JNIEnv,
    _class: JClass,
    path: JString,
    create_if_missing: bool,
) -> jlong {
    let path_str = match env.get_string(path) {
        Ok(path) => path,
        Err(_) => return 0, // Error
    };

    let path_native: String = path_str.into();
    let path = std::path::Path::new(&path_native);

    let mut options = Options::default();
    options.create_if_missing(create_if_missing);

    match DB::open(&options, path) {
        Ok(db) => db_to_handle(db),
        Err(_) => 0, // Error
    }
}

#[cfg(feature = "java-bindings")]
#[no_mangle]
pub extern "system" fn Java_org_rocksdb_ToplingDB_close(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        unsafe {
            release_db_handle(handle);
        }
    }
}

#[cfg(feature = "java-bindings")]
#[no_mangle]
pub extern "system" fn Java_org_rocksdb_ToplingDB_put(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    key: jbyteArray,
    value: jbyteArray,
) -> bool {
    if handle == 0 {
        return false;
    }

    let db = unsafe { handle_to_db(handle) };

    let key_bytes = match env.convert_byte_array(key) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let value_bytes = match env.convert_byte_array(value) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let write_options = WriteOptions::default();
    match db.put(&write_options, &key_bytes, &value_bytes) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(feature = "java-bindings")]
#[no_mangle]
pub extern "system" fn Java_org_rocksdb_ToplingDB_get(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    key: jbyteArray,
) -> jbyteArray {
    if handle == 0 {
        return ptr::null_mut();
    }

    let db = unsafe { handle_to_db(handle) };

    let key_bytes = match env.convert_byte_array(key) {
        Ok(bytes) => bytes,
        Err(_) => return ptr::null_mut(),
    };

    let read_options = ReadOptions::default();
    match db.get(&read_options, &key_bytes) {
        Ok(Some(value)) => {
            match env.byte_array_from_slice(&value) {
                Ok(array) => array,
                Err(_) => ptr::null_mut(),
            }
        }
        Ok(None) => ptr::null_mut(),
        Err(_) => ptr::null_mut(),
    }
}

#[cfg(feature = "java-bindings")]
#[no_mangle]
pub extern "system" fn Java_org_rocksdb_ToplingDB_delete(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    key: jbyteArray,
) -> bool {
    if handle == 0 {
        return false;
    }

    let db = unsafe { handle_to_db(handle) };

    let key_bytes = match env.convert_byte_array(key) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let write_options = WriteOptions::default();
    match db.delete(&write_options, &key_bytes) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(feature = "java-bindings")]
#[no_mangle]
pub extern "system" fn Java_org_rocksdb_ToplingDB_flush(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> bool {
    if handle == 0 {
        return false;
    }

    let db = unsafe { handle_to_db(handle) };
    let flush_options = crate::options::FlushOptions::default();
    match db.flush(&flush_options) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(feature = "java-bindings")]
#[no_mangle]
pub extern "system" fn Java_org_rocksdb_ToplingDB_getProperty(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    property: JString,
) -> jstring {
    if handle == 0 {
        return ptr::null_mut();
    }

    let db = unsafe { handle_to_db(handle) };

    let property_str = match env.get_string(property) {
        Ok(prop) => prop,
        Err(_) => return ptr::null_mut(),
    };

    let property_native: String = property_str.into();

    match db.get_property(&property_native) {
        Some(value) => {
            match env.new_string(value) {
                Ok(jstr) => jstr.into_inner(),
                Err(_) => ptr::null_mut(),
            }
        }
        None => ptr::null_mut(),
    }
}

// Java class template that would be used with these bindings
#[cfg(feature = "java-bindings")]
pub const JAVA_CLASS_TEMPLATE: &str = r#"
package org.rocksdb;

public class ToplingDB {
    static {
        System.loadLibrary("toplingdb");
    }

    private long nativeHandle;

    private ToplingDB(long handle) {
        this.nativeHandle = handle;
    }

    public static ToplingDB open(String path, boolean createIfMissing) throws Exception {
        long handle = open(path, createIfMissing);
        if (handle == 0) {
            throw new Exception("Failed to open database");
        }
        return new ToplingDB(handle);
    }

    public void put(byte[] key, byte[] value) throws Exception {
        if (!put(nativeHandle, key, value)) {
            throw new Exception("Failed to put key-value pair");
        }
    }

    public byte[] get(byte[] key) {
        return get(nativeHandle, key);
    }

    public void delete(byte[] key) throws Exception {
        if (!delete(nativeHandle, key)) {
            throw new Exception("Failed to delete key");
        }
    }

    public void flush() throws Exception {
        if (!flush(nativeHandle)) {
            throw new Exception("Failed to flush");
        }
    }

    public String getProperty(String property) {
        return getProperty(nativeHandle, property);
    }

    public void close() {
        if (nativeHandle != 0) {
            close(nativeHandle);
            nativeHandle = 0;
        }
    }

    @Override
    protected void finalize() throws Throwable {
        close();
        super.finalize();
    }

    // Native methods
    private static native long open(String path, boolean createIfMissing);
    private static native void close(long handle);
    private static native boolean put(long handle, byte[] key, byte[] value);
    private static native byte[] get(long handle, byte[] key);
    private static native boolean delete(long handle, byte[] key);
    private static native boolean flush(long handle);
    private static native String getProperty(long handle, String property);
}
"#;

// Provide stub implementations when java-bindings feature is not enabled
#[cfg(not(feature = "java-bindings"))]
pub const JAVA_CLASS_TEMPLATE: &str = r#"
// Java bindings not available. Enable the 'java-bindings' feature to use Java bindings.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_class_template() {
        assert!(!JAVA_CLASS_TEMPLATE.is_empty());
    }
}