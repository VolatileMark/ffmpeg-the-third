use std::mem;
use std::ops::{Bound, Deref, RangeBounds};

use crate::util::error::Error;

use super::Stream;
use crate::ffi::*;
use crate::format::context::common::Context;
use crate::{codec, Dictionary, Rational};

pub struct StreamMut<'a> {
    context: &'a mut Context,
    index: usize,

    immutable: Stream<'a>,
}

impl<'a> StreamMut<'a> {
    pub unsafe fn wrap(context: &mut Context, index: usize) -> StreamMut {
        StreamMut {
            context: mem::transmute_copy(&context),
            index,

            immutable: Stream::wrap(mem::transmute_copy(&context), index),
        }
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVStream {
        *(*self.context.as_mut_ptr()).streams.add(self.index)
    }
}

impl<'a> StreamMut<'a> {
    pub fn set_time_base<R: Into<Rational>>(&mut self, value: R) {
        unsafe {
            (*self.as_mut_ptr()).time_base = value.into().into();
        }
    }

    pub fn set_rate<R: Into<Rational>>(&mut self, value: R) {
        unsafe {
            (*self.as_mut_ptr()).r_frame_rate = value.into().into();
        }
    }

    pub fn set_avg_frame_rate<R: Into<Rational>>(&mut self, value: R) {
        unsafe {
            (*self.as_mut_ptr()).avg_frame_rate = value.into().into();
        }
    }

    pub fn set_parameters<P: Into<codec::Parameters>>(&mut self, parameters: P) {
        let parameters = parameters.into();

        unsafe {
            avcodec_parameters_copy((*self.as_mut_ptr()).codecpar, parameters.as_ptr());
        }
    }

    pub fn set_metadata(&mut self, metadata: Dictionary) {
        unsafe {
            let metadata = metadata.disown();
            (*self.as_mut_ptr()).metadata = metadata;
        }
    }

    pub fn seek<R: RangeBounds<i64>>(&mut self, ts: i64, range: R) -> Result<(), Error> {
        unsafe {
            let start = match range.start_bound().cloned() {
                Bound::Included(i) => i,
                Bound::Excluded(i) => i.saturating_add(1),
                Bound::Unbounded => i64::MIN,
            };

            let end = match range.end_bound().cloned() {
                Bound::Included(i) => i,
                Bound::Excluded(i) => i.saturating_sub(1),
                Bound::Unbounded => i64::MAX,
            };

            match avformat_seek_file(
                self.context.as_mut_ptr(),
                self.index as _,
                start,
                ts,
                end,
                0,
            ) {
                s if s >= 0 => Ok(()),
                e => Err(Error::from(e)),
            }
        }
    }
}

impl<'a> Deref for StreamMut<'a> {
    type Target = Stream<'a>;

    fn deref(&self) -> &Self::Target {
        &self.immutable
    }
}
