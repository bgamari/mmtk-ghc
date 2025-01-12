// use crate::DummyVM;
use super::types::*;
use super::stg_closures::*;
use std::fmt;
/**
 * GHC closure info tables in Rust
 * Original C code is at ghc/rts/include/rts/storage/InfoTables.h
 */

/* -----------------------------------------------------------------------------
   Closure flags
   -------------------------------------------------------------------------- */

#[repr(C)]
pub struct ClosureFlag (StgWord16);

impl ClosureFlag {
    const _HNF : ClosureFlag = ClosureFlag(1<<0);  /* head normal form?    */
    const _BTM : ClosureFlag = ClosureFlag(1<<1);  /* uses info->layout.bitmap */
    const _NS  : ClosureFlag = ClosureFlag(1<<2);  /* non-sparkable        */
    const _THU : ClosureFlag = ClosureFlag(1<<3);  /* thunk?               */
    const _MUT : ClosureFlag = ClosureFlag(1<<4);  /* mutable?             */
    const _UPT : ClosureFlag = ClosureFlag(1<<5);  /* unpointed?           */
    const _SRT : ClosureFlag = ClosureFlag(1<<6);  /* has an SRT?          */
    const _IND : ClosureFlag = ClosureFlag(1<<7);  /* is an indirection?   */

    #[inline(always)]
    pub fn is_mutable(&self)     -> bool {(self.0) & (Self::_MUT.0) != 0}

    #[inline(always)]
    pub fn is_bitmap(&self)      -> bool {(self.0) & (Self::_BTM.0) != 0}

    #[inline(always)]
    pub fn is_thunk(&self)       -> bool {(self.0) & (Self::_THU.0) != 0}

    #[inline(always)]
    pub fn is_unpointed(&self)   -> bool {(self.0) & (Self::_UPT.0) != 0}

    #[inline(always)]
    pub fn has_srt(&self)        -> bool {(self.0) & (Self::_SRT.0) != 0}
    

    // TODO: implement closure flags related macros
    #[inline(always)]
    pub fn get_closure_flag(_c : *const StgClosure) -> ClosureFlag {
        unimplemented!()
    }

}


/* -----------------------------------------------------------------------------
   Bitmaps
   -------------------------------------------------------------------------- */
pub union Bitmap {
    pub small_bitmap        : StgSmallBitmap,
    pub large_bitmap_ref   : StgLargeBitmapRef,
}

// -------------------- small bitmap --------------------
#[repr(C)]
pub struct StgSmallBitmap (StgWord);

impl StgSmallBitmap {
    // TODO: handle 32 bits constants
    const BITMAP_BITS_SHIFT : StgWord = 6;
    const BITMAP_SIZE_MASK  : StgWord = 0x3f;
    // const BITMAP_BITS_SHIFT : StgWord = 6;

    #[inline(always)]
    pub fn make_small_bitmap(size : StgWord, bits : StgWord) -> Self {
        StgSmallBitmap(((bits) << Self::BITMAP_BITS_SHIFT) | (size))
    }

    #[inline(always)]
    pub fn size(&self) -> StgWord {
        (self.0) & Self::BITMAP_SIZE_MASK 
    }

    #[inline(always)]
    pub fn bits(&self) -> StgWord {
        (self.0) >> Self::BITMAP_BITS_SHIFT
    }
}

// -------------------- large bitmap --------------------

#[repr(C)]
#[derive(Debug)]
pub struct StgLargeBitmap {
    pub size    : StgWord,
    pub bitmap  : LargeBitMapPayload // similar to closure payload in stg_closures.rs
}

#[repr(C)]
#[derive(Debug)]
pub struct LargeBitMapPayload {}

impl LargeBitMapPayload {
    pub fn get_w(&self, i: usize) -> *mut StgClosure {
        unsafe {
            let ptr: *const LargeBitMapPayload = &*self;
            let payload: *const *mut StgClosure = ptr.cast();
            *payload.offset(i as isize)
        }
    }
    // TODO: might want to iterate through bits as well
}

#[repr(C)]
pub struct StgLargeBitmapRef {
    pub offset : StgInt
    // TODO: handle non TABLES_NEXT_TO_CODE
}

impl StgLargeBitmapRef {
    pub fn deref(&self, itbl: &StgInfoTable) -> *const StgLargeBitmap {
        unsafe {
            offset_from_end(itbl, self.offset as isize)
        }
    }
}


/* ----------------------------------------------------------------------------
   Info Tables
   ------------------------------------------------------------------------- */
#[repr(C)]
pub struct StgPointerFirst {
    pub ptrs    : StgHalfWord,  /* number of pointers */
    pub nptrs   : StgHalfWord,  /* number of non-pointers */
}

#[repr(C)]
pub union StgClosureInfo {
    pub payload : StgPointerFirst,

    pub small_bitmap : StgSmallBitmap,
    
    // TODO: check if x64 is still related to OFFSET_FIELD
    // Check if hack in Note [x86-64-relative] is still necessary 
    pub large_bitmap : StgLargeBitmapRef,

    pub selector_offset : StgWord,
}

impl fmt::Debug for StgClosureInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StgClosureInfo")
    }
}

/* ----------------------------------------------------------------------------
   Function info tables
   ------------------------------------------------------------------------- */

#[repr(C)]
#[derive(Debug)]
pub struct StgSRTField {
    pub srt : StgHalfInt,
    // TODO: handle non USE_INLINE_SRT_FIELD
}

#[cfg(not(profiling))]
#[repr(C)]
#[derive(Debug)]
pub struct StgProfInfo {} // TODO: handle profiling case

#[repr(C)]
#[derive(Debug)]
pub struct StgInfoTable {
    // TODO: non TABLES_NEXT_TO_CODE
    #[cfg(not(tables_next_to_code))]
    pub code    : *const u8, // pointer to entry code
    pub prof    : StgProfInfo,
    pub layout  : StgClosureInfo,
    pub type_   : StgClosureType,
    pub srt     : StgSRTField, // what to do with SRT?
    // pub code    : *mut StgCode, (zero length array)
}


#[repr(C)]
pub struct StgFunInfoExtra {
    pub slow_apply  : StgInt,
    pub bitmap      : Bitmap,

    // TODO: handle offset for USE_INLINE_SRT_FIELD for srtfield

    pub fun_type    : StgFunType, // in types.rs from rts/include/rts/storage/FunTypes.h
    pub arity       : StgHalfWord,
    // TODO: handle non TABLES_NEXT_TO_CODE (StgFunInfoExtraFwd)
}

#[repr(C)]
pub struct StgFunInfoTable {
    pub f : StgFunInfoExtra,
    pub i : StgInfoTable
    // TODO: handle non TABLES_NEXT_TO_CODE (need to use StgFunInfoExtraFwd)
}

/* -----------------------------------------------------------------------------
   Return info tables
   -------------------------------------------------------------------------- */
#[repr(C)]
pub struct StgRetInfoTable {
    // (check line 160 InfoTables.h)
    // TODO: USE_SRT_POINTER is true
    // TODO: USE_SRT_POINTER is false but USE_SRT_OFFSET is true
    pub i : StgInfoTable, // both false case
}

impl StgRetInfoTable {
    pub fn get_srt(&self) -> *const StgClosure {
        unsafe {
            offset_from_end(self, self.i.srt.srt as isize)
        }
    }
}

/// Compute a pointer to a structure from an offset relative
/// to the end of another structure.
unsafe fn offset_from_end<Src, Target>(ptr: &Src, offset: isize) -> *const Target {
    let end = (ptr as *const Src).offset(1);
    (end as *const u8).offset(offset).cast()
}
/* -----------------------------------------------------------------------------
   Thunk info tables
   -------------------------------------------------------------------------- */
#[repr(C)]
pub struct StgThunkInfoTable {
    // (check line 160 InfoTables.h)
    // TODO: USE_SRT_POINTER is true
    // TODO: USE_SRT_POINTER is false but USE_SRT_OFFSET is true
    pub i : StgInfoTable, // both false case
}

impl StgThunkInfoTable {
    pub fn get_srt(&self) -> *const StgClosure {
        unsafe {
            offset_from_end(self, self.i.srt.srt as isize)
        }
    }
}

/* -----------------------------------------------------------------------------
   Constructor info tables
   -------------------------------------------------------------------------- */
#[repr(C)]
pub struct StgConInfoTable {
    // TODO: handle non TABLES_NEXT_TO_CODE
    pub con_desc_offset : StgInt,
    pub i               : StgInfoTable,
}

// TODO: implement other macros

#[no_mangle]
pub extern "C" fn print_obj(obj : TaggedClosureRef){
    unsafe{
        println!("{:?}", Closure::from_ptr(obj.to_ptr()));
    }
}