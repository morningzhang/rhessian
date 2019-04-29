extern crate bytes;


#[cfg(test)]
mod hessian {
    mod decode {
        use std::any::Any;
        use std::str;
        use bytes::{Buf, IntoBuf};

        fn read_object(buf: &mut Buf) -> Option<Box<Any>> {
            let code = buf.get_u8();
            //none
            if code == b'N' {
                return None;
            }
            //bool
            else if code == b'T' {
                return Some(Box::new(true));
            } else if code == b'F' {
                return Some(Box::new(false));
            }
            //int
            else if code == b'I' {
                return Some(Box::new(buf.get_i32_be()));
            } else if 0x80 <= code && code <= 0xbf {
                let code = code as i16;
                return Some(Box::new(code - 0x90));
            } else if 0xc0 <= code && code <= 0xcf {
                let code = code as i16;
                let b0 = buf.get_u8() as i16;
                return Some(Box::new(((code - 0xc8) << 8) + b0));
            } else if 0xd0 <= code && code <= 0xd7 {
                let code = code as i32;
                let b1 = buf.get_u8() as i32;
                let b0 = buf.get_u8() as i32;
                return Some(Box::new(((code - 0xd4) << 16) + (b1 << 8) + b0));
            }
            //double
            else if code == b'D' {
                return Some(Box::new(buf.get_f64_be()));
            } else if code == 0x5b {
                return Some(Box::new(0.0f32));
            } else if code == 0x5c {
                return Some(Box::new(1.0f32));
            } else if code == 0x5d {
                let b0 = buf.get_i8() as f32;
                return Some(Box::new(b0));
            } else if code == 0x5e {
                let b1 = buf.get_i8() as f32;
                let b0 = buf.get_u8() as f32;
                return Some(Box::new(256.0 * b1 + b0));
            } else if code == 0x5f {
                return Some(Box::new(buf.get_f32_be()));
            }

            //long
            else if code == b'L' {
                return Some(Box::new(buf.get_i64_be()));
            } else if 0xd8 <= code && code <= 0xef {
                let code = code as i16;
                return Some(Box::new(code - 0xe0));
            } else if 0xf0 <= code && code <= 0xff {
                let code = code as i16;
                let b0 = buf.get_u8() as i16;
                return Some(Box::new(((code - 0xf8) << 8) + b0));
            } else if 0x38 <= code && code <= 0x3f {
                let code = code as i32;
                let b1 = buf.get_u8() as i32;
                let b0 = buf.get_u8() as i32;
                return Some(Box::new(((code - 0x3c) << 16) + (b1 << 8) + b0));
            } else if code == 0x59 {//NOT 0x4c
                return Some(Box::new(buf.get_i32_be()));
            }
            //date
            else if code == 0x4a {
                return Some(Box::new(buf.get_i64_be()));
            } else if code == 0x4b {
                return Some(Box::new(buf.get_i32_be()));
            }

            //string
            else if 0x00 <= code && code <= 0x1f {
                let s = buf.bytes().get(..code as usize).unwrap();
                //buf.advance(code as usize);
                return Some(Box::new(str::from_utf8(s).unwrap().to_string()));
            } else if 0x30 <= code && code <= 0x33 {
                let b0 = buf.get_u8();
                let s = buf.bytes().get(..b0 as usize).unwrap();
                //buf.advance(b0 as usize);
                return Some(Box::new(str::from_utf8(s).unwrap().to_string()));
            } else if code == b'S' {
                let str_len = buf.get_u16_be();
                let s = buf.bytes().get(..str_len as usize).unwrap();
                //buf.advance(str_len as usize);
                return Some(Box::new(str::from_utf8(s).unwrap().to_string()));
            } else if code == b'R' {
                let str_len = buf.get_u16_be();
                let s = buf.bytes().get(..str_len as usize).unwrap();
                //buf.advance(str_len as usize);

                let mut s_str =s.to_vec();

                let mut res_str = vec![];
                res_str.append(&mut s_str);

                loop {
                    let (is_ending,_other_str_len,  other_part) = read_string(buf.bytes()[0], buf);
                    if other_part.is_none() == false {

                        res_str.append(&mut other_part.unwrap().to_vec());
                    }
                    if is_ending == true {
                        break;
                    }
                };
                return Some(Box::new(str::from_utf8(res_str.as_slice()).unwrap().to_string()));
            }


            return None;
        }



        fn read_string(code: u8, buf: & mut Buf) -> (bool, u16, Option<&[u8]>) {
            if 0x00 <= code && code <= 0x1f {
                buf.advance(1);
                let s = buf.bytes().get(..code as usize).unwrap();
               // buf.advance(code as usize);
                return (true, code as u16, Some(s));
            } else if 0x30 <= code && code <= 0x33 {
                buf.advance(1);
                let b0 = buf.get_u8();
                let s = buf.bytes().get(..b0 as usize).unwrap();
                //buf.advance(b0 as usize);
                return (true, b0 as u16, Some(s));
            } else if code == b'S' {
                buf.advance(1);
                let str_len = buf.get_u16_be();
                let s = buf.bytes().get(..str_len as usize).unwrap();
                //buf.advance(str_len as usize);
                return (true, str_len as u16, Some(s));
            } else if code == b'R' {
                buf.advance(1);
                let str_len = buf.get_u16_be();
                let s = buf.bytes().get(..str_len as usize).unwrap();
               // buf.advance(str_len as usize);
                return (false, str_len as u16, Some(s));
            }

            return (true, 0, None);
        }


        #[test]
        fn test_bool() {
            let mut buf = b"T".into_buf();
            assert_eq!(true, *read_object(&mut buf).unwrap().downcast_ref::<bool>().unwrap());

            let mut buf = b"F".into_buf();
            assert_eq!(false, *read_object(&mut buf).unwrap().downcast_ref::<bool>().unwrap());
        }

        #[test]
        fn test_int() {
            //4byte
            let mut buf = b"I\x00\x00\x01\x2c".into_buf();
            assert_eq!(300, *read_object(&mut buf).unwrap().downcast_ref::<i32>().unwrap());

            //1byte
            let tests = vec![(b"\x90", 0), (b"\x80", -16), (b"\xbf", 47)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<i16>().unwrap());
            }
            //2byte
            let tests = vec![(b"\xc8\x00", 0), (b"\xc0\x00", -2048), (b"\xc7\x00", -256), (b"\xcf\xff", 2047)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<i16>().unwrap());
            }
            //3byte
            let tests = vec![(b"\xd4\x00\x00", 0), (b"\xd0\x00\x00", -262144), (b"\xd7\xff\xff", 262143)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<i32>().unwrap());
            }
        }


        #[test]
        fn test_double() {
            //1byte
            let tests = vec![(b"\x5b", 0.0), (b"\x5c", 1.0)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<f32>().unwrap());
            }
            //2byte
            let tests = vec![(b"\x5d\x80", -128.0), (b"\x5d\x7f", 127.0)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<f32>().unwrap());
            }
            //3byte
            let tests = vec![(b"\x5e\x00\x00", 0.0), (b"\x5e\x80\x00", -32768.0), (b"\x5e\x7f\xff", 32767.0)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<f32>().unwrap());
            }

            //8byte
            let tests = vec![(b"D\x40\x28\x80\x00\x00\x00\x00\x00", 12.25)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<f64>().unwrap());
            }
        }

        #[test]
        fn test_long() {
            //1byte
            let tests = vec![(b"\xe0", 0), (b"\xd8", -8), (b"\xef", 15)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<i16>().unwrap());
            }

            //2byte
            let tests = vec![(b"\xf8\x00", 0), (b"\xf0\x00", -2048), (b"\xf7\x00", -256), (b"\xff\xff", 2047)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<i16>().unwrap());
            }

            //3byte
            let tests = vec![(b"\x3c\x00\x00", 0), (b"\x38\x00\x00", -262144), (b"\x3f\xff\xff", 262143)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<i32>().unwrap());
            }
            //8byte
            let tests = vec![(b"L\x00\x00\x00\x00\x00\x00\x01\x2c", 300)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<i64>().unwrap());
            }

            //4byte
            let tests = vec![(b"\x59\x00\x00\x00\x00", 0), (b"\x59\x00\x00\x01\x2c", 300)];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<i32>().unwrap());
            }
        }


        #[test]
        fn test_string() {
            let tests = vec![(b"\x00", "")];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                assert_eq!(t.1, *read_object(&mut buf).unwrap().downcast_ref::<String>().unwrap());
            }
            let tests = vec![("\x05我".as_bytes(), "我")];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                let obj = read_object(&mut buf).unwrap();
                let s = obj.downcast_ref::<String>().unwrap();
                assert_eq!(t.1, *s);
            }


            let tests = vec![(b"\x01\xc3\x83", "\\u00c3")];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                let obj = read_object(&mut buf).unwrap();
                let s = obj.downcast_ref::<String>().unwrap();
                assert_eq!(t.1, *s);
            }

            let tests = vec![(b"\x52\x00\x07hello, \x05world", "hello, world")];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                let obj = read_object(&mut buf).unwrap();
                let s = obj.downcast_ref::<String>().unwrap();
                assert_eq!(t.1, *s);
            }
        }

        #[test]
        fn test_string1() {
            let tests = vec![(b"\x01\xc3\x83", "\\u00c3")];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                let obj = read_object(&mut buf).unwrap();
                let s = obj.downcast_ref::<String>().unwrap();
                assert_eq!(t.1, *s);
            }
        }

        #[test]
        fn test_string2() {
            let tests = vec![(b"\x52\x00\x07hello, \x05world", "hello, world")];
            for t in tests.iter() {
                let mut buf = t.0.into_buf();
                let obj = read_object(&mut buf).unwrap();
                let s = obj.downcast_ref::<String>().unwrap();
                assert_eq!(t.1, *s);
            }
        }
    }
}

