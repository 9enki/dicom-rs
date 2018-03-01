//! Explicit VR Big Endian syntax transfer implementation.

use std::io::{Read, Write};
use std::fmt;
use std::marker::PhantomData;
use data::VR;
use data::Tag;
use byteorder::{BigEndian, ByteOrder};
use error::Result;
use data::decode::basic::BigEndianBasicDecoder;
use data::decode::{BasicDecode, Decode};
use data::encode::{BasicEncode, Encode};
use data::encode::basic::BigEndianBasicEncoder;
use util::Endianness;
use data::{DataElementHeader, Header, SequenceItemHeader};

/// A data element decoder for the Explicit VR Big Endian transfer syntax.
pub struct ExplicitVRBigEndianDecoder<S: ?Sized> {
    basic: BigEndianBasicDecoder,
    phantom: PhantomData<S>,
}

impl<S: ?Sized> Default for ExplicitVRBigEndianDecoder<S> {
    fn default() -> ExplicitVRBigEndianDecoder<S> {
        ExplicitVRBigEndianDecoder {
            basic: BigEndianBasicDecoder,
            phantom: PhantomData,
        }
    }
}

impl<S: ?Sized> fmt::Debug for ExplicitVRBigEndianDecoder<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExplicitVRLittleEndianDecoder")
            .field("basic", &self.basic)
            .finish()
    }
}

impl<S: ?Sized> Decode for ExplicitVRBigEndianDecoder<S>
where
    S: Read,
{
    type Source = S;

    fn decode_header(&self, mut source: &mut Self::Source) -> Result<DataElementHeader> {
        // retrieve tag
        let tag = self.basic.decode_tag(&mut source)?;

        let mut buf = [0u8; 4];
        // retrieve explicit VR
        source.read_exact(&mut buf[0..2])?;
        let vr = VR::from_binary([buf[0], buf[1]]).unwrap_or(VR::UN);

        // retrieve data length
        let len = match vr {
            VR::OB
            | VR::OD
            | VR::OF
            | VR::OL
            | VR::OW
            | VR::SQ
            | VR::UC
            | VR::UR
            | VR::UT
            | VR::UN => {
                // read 2 reserved bytes, then 4 bytes for data length
                source.read_exact(&mut buf[0..2])?;
                source.read_exact(&mut buf)?;
                BigEndian::read_u32(&buf)
            }
            _ => {
                // read 2 bytes for the data length
                source.read_exact(&mut buf[0..2])?;
                BigEndian::read_u16(&buf[0..2]) as u32
            }
        };

        Ok(DataElementHeader::new(tag, vr, len))
    }

    fn decode_item_header(&self, mut source: &mut Self::Source) -> Result<SequenceItemHeader> {
        let mut buf = [0u8; 4];
        // retrieve tag
        let tag = self.basic.decode_tag(&mut source)?;
        // and item sequence length
        source.read_exact(&mut buf)?;
        let len = BigEndian::read_u32(&buf);

        SequenceItemHeader::new(tag, len)
    }

    fn decode_tag(&self, source: &mut Self::Source) -> Result<Tag> {
        let mut buf = [0u8; 4];
        source.read_exact(&mut buf)?;
        Ok(Tag(
            BigEndian::read_u16(&buf[0..2]),
            BigEndian::read_u16(&buf[2..4]),
        ))
    }
}

/// A concrete encoder for the transfer syntax ExplicitVRBigEndian
pub struct ExplicitVRBigEndianEncoder<W: ?Sized> {
    basic: BigEndianBasicEncoder,
    phantom: PhantomData<W>,
}

impl<W: ?Sized> Default for ExplicitVRBigEndianEncoder<W> {
    fn default() -> ExplicitVRBigEndianEncoder<W> {
        ExplicitVRBigEndianEncoder {
            basic: BigEndianBasicEncoder,
            phantom: PhantomData,
        }
    }
}

impl<W: ?Sized + fmt::Debug> fmt::Debug for ExplicitVRBigEndianEncoder<W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExplicitVRLittleEndianDecoder")
            .field("basic", &self.basic)
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<W: ?Sized> BasicEncode for ExplicitVRBigEndianEncoder<W> {
    fn endianness(&self) -> Endianness {
        Endianness::BE
    }

    fn encode_us<S>(&self, to: S, value: u16) -> Result<()>
    where
        S: Write,
    {
        self.basic.encode_us(to, value)
    }

    fn encode_ul<S>(&self, to: S, value: u32) -> Result<()>
    where
        S: Write,
    {
        self.basic.encode_ul(to, value)
    }

    fn encode_ss<S>(&self, to: S, value: i16) -> Result<()>
    where
        S: Write,
    {
        self.basic.encode_ss(to, value)
    }

    fn encode_sl<S>(&self, to: S, value: i32) -> Result<()>
    where
        S: Write,
    {
        self.basic.encode_sl(to, value)
    }

    fn encode_fl<S>(&self, to: S, value: f32) -> Result<()>
    where
        S: Write,
    {
        self.basic.encode_fl(to, value)
    }

    fn encode_fd<S>(&self, to: S, value: f64) -> Result<()>
    where
        S: Write,
    {
        self.basic.encode_fd(to, value)
    }
}

impl<W: ?Sized> Encode for ExplicitVRBigEndianEncoder<W>
where
    W: Write,
{
    type Writer = W;

    fn encode_tag(&self, to: &mut W, tag: Tag) -> Result<()> {
        let mut buf = [0u8, 4];
        BigEndian::write_u16(&mut buf[..], tag.group());
        BigEndian::write_u16(&mut buf[2..], tag.element());
        to.write_all(&buf)?;
        Ok(())
    }

    fn encode_element_header(&self, to: &mut W, de: DataElementHeader) -> Result<usize> {
        match de.vr() {
            VR::OB
            | VR::OD
            | VR::OF
            | VR::OL
            | VR::OW
            | VR::SQ
            | VR::UC
            | VR::UR
            | VR::UT
            | VR::UN => {
                let mut buf = [0u8; 12];
                BigEndian::write_u16(&mut buf[0..], de.tag().group());
                BigEndian::write_u16(&mut buf[2..], de.tag().element());
                let vr_bytes = de.vr().to_bytes();
                buf[4] = vr_bytes[0];
                buf[5] = vr_bytes[1];
                // buf[6..8] is kept zero'd
                BigEndian::write_u32(&mut buf[8..], de.len());
                try!(to.write_all(&buf));

                Ok(12)
            }
            _ => {
                let mut buf = [0u8; 8];
                BigEndian::write_u16(&mut buf[0..], de.tag().group());
                BigEndian::write_u16(&mut buf[2..], de.tag().element());
                let vr_bytes = de.vr().to_bytes();
                buf[4] = vr_bytes[0];
                buf[5] = vr_bytes[1];
                BigEndian::write_u16(&mut buf[6..], de.len() as u16);
                try!(to.write_all(&buf));

                Ok(8)
            }
        }
    }

    fn encode_item_header(&self, to: &mut W, len: u32) -> Result<()> {
        let mut buf = [0u8; 8];
        BigEndian::write_u16(&mut buf, 0xFFFE);
        BigEndian::write_u16(&mut buf, 0xE000);
        BigEndian::write_u32(&mut buf[4..], len);
        to.write_all(&buf)?;
        Ok(())
    }

    fn encode_item_delimiter(&self, to: &mut W) -> Result<()> {
        let mut buf = [0u8; 8];
        BigEndian::write_u16(&mut buf, 0xFFFE);
        BigEndian::write_u16(&mut buf, 0xE00D);
        // remaining bytes are already zero, so it's ready to write
        to.write_all(&buf)?;
        Ok(())
    }

    fn encode_sequence_delimiter(&self, to: &mut W) -> Result<()> {
        let mut buf = [0u8; 8];
        BigEndian::write_u16(&mut buf, 0xFFFE);
        BigEndian::write_u16(&mut buf, 0xE0DD);
        // remaining bytes are already zero, so it's ready to write
        to.write_all(&buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use data::{DataElementHeader, Header};
    use data::decode::Decode;
    use data::encode::Encode;
    use super::ExplicitVRBigEndianDecoder;
    use super::ExplicitVRBigEndianEncoder;
    use data::VR;
    use data::Tag;
    use std::io::{Cursor, Read, Seek, SeekFrom, Write};

    // manually crafting some DICOM data elements
    //  Tag: (0002,0002) Media Storage SOP Class UID
    //  VR: UI
    //  Length: 26
    //  Value: "1.2.840.10008.5.1.4.1.1.1" (with 1 padding '\0')
    // --
    //  Tag: (0002,0010) Transfer Syntax UID
    //  VR: UI
    //  Length: 20
    //  Value: "1.2.840.10008.1.2.1" (w 1 padding '\0') == ExplicitVRLittleEndian
    // --
    const RAW: &'static [u8; 62] = &[
        0x00, 0x02, 0x00, 0x02, 0x55, 0x49, 0x00, 0x1a, 0x31, 0x2e, 0x32, 0x2e, 0x38, 0x34, 0x30,
        0x2e, 0x31, 0x30, 0x30, 0x30, 0x38, 0x2e, 0x35, 0x2e, 0x31, 0x2e, 0x34, 0x2e, 0x31, 0x2e,
        0x31, 0x2e, 0x31, 0x00, 0x00, 0x02, 0x00, 0x10, 0x55, 0x49, 0x00, 0x14, 0x31, 0x2e, 0x32,
        0x2e, 0x38, 0x34, 0x30, 0x2e, 0x31, 0x30, 0x30, 0x30, 0x38, 0x2e, 0x31, 0x2e, 0x32, 0x2e,
        0x31, 0x00,
    ];

    #[test]
    fn decode_explicit_vr_be_works() {
        let reader = ExplicitVRBigEndianDecoder::default();
        let mut cursor = Cursor::new(RAW.as_ref());
        {
            // read first element
            let elem = reader
                .decode_header(&mut cursor)
                .expect("should find an element");
            assert_eq!(elem.tag(), Tag(2, 2));
            assert_eq!(elem.vr(), VR::UI);
            assert_eq!(elem.len(), 26);
            // read only half of the data
            let mut buffer: Vec<u8> = Vec::with_capacity(13);
            buffer.resize(13, 0);
            cursor
                .read_exact(buffer.as_mut_slice())
                .expect("should read it fine");
            assert_eq!(buffer.as_slice(), b"1.2.840.10008".as_ref());
        }
        // cursor should now be @ #21 (there is no automatic skipping)
        assert_eq!(cursor.seek(SeekFrom::Current(0)).unwrap(), 21);
        // cursor should now be @ #34 after skipping
        assert_eq!(cursor.seek(SeekFrom::Current(13)).unwrap(), 34);
        {
            // read second element
            let elem = reader
                .decode_header(&mut cursor)
                .expect("should find an element");
            assert_eq!(elem.tag(), Tag(2, 16));
            assert_eq!(elem.vr(), VR::UI);
            assert_eq!(elem.len(), 20);
            // read all data
            let mut buffer: Vec<u8> = Vec::with_capacity(20);
            buffer.resize(20, 0);
            cursor
                .read_exact(buffer.as_mut_slice())
                .expect("should read it fine");
            assert_eq!(buffer.as_slice(), b"1.2.840.10008.1.2.1\0".as_ref());
        }
    }

    #[test]
    fn encode_explicit_vr_be_works() {
        let mut buf = [0u8; 62];
        {
            let enc = ExplicitVRBigEndianEncoder::default();
            let mut writer = Cursor::new(&mut buf[..]);

            // encode first element
            let de = DataElementHeader::new(Tag(0x0002, 0x0002), VR::UI, 26);
            let len = enc.encode_element_header(&mut writer, de)
                .expect("should write it fine");
            assert_eq!(len, 8);
            writer
                .write_all(b"1.2.840.10008.5.1.4.1.1.1\0".as_ref())
                .expect("should write the value fine");
        }
        assert_eq!(&buf[0..8], &RAW[0..8]);
        {
            let enc = ExplicitVRBigEndianEncoder::default();
            let mut writer = Cursor::new(&mut buf[34..]);

            // encode second element
            let de = DataElementHeader::new(Tag(0x0002, 0x0010), VR::UI, 20);
            let len = enc.encode_element_header(&mut writer, de)
                .expect("should write it fine");
            assert_eq!(len, 8);
            writer
                .write_all(b"1.2.840.10008.1.2.1\0".as_ref())
                .expect("should write the value fine");
        }
        assert_eq!(&buf[34..42], &RAW[34..42]);

        assert_eq!(&buf[..], &RAW[..]);
    }
}