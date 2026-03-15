use ffmpeg::{
    codec, decoder, encoder, format, media,
    software::{resampling, scaling},
    Dictionary, Rational, Rescale,
};
use ffmpeg_next as ffmpeg;

use crate::structs::{Video, VideoStats};

#[cfg(target_os = "android")]
const PIXEL_FORMAT: format::Pixel = format::Pixel::NV12;
#[cfg(not(target_os = "android"))]
const PIXEL_FORMAT: format::Pixel = format::Pixel::YUV420P;

fn findCodec() -> Option<codec::codec::Codec> {
    #[cfg(target_os = "android")]
    return encoder::find(codec::Id::H264);

    #[cfg(not(target_os = "android"))]
    return encoder::find_by_name("libx264");
}

fn makeVideoEncoder(
    ffOut: &mut format::context::Output,
    width: u32,
    height: u32,
    frameRate: Rational,
    crf: Option<u8>,
) -> Result<encoder::Video, String> {
    println!(
        "Setting up video encoder with width: {}, height: {}, frame rate: {}/{}, crf: {}",
        width,
        height,
        frameRate.numerator(),
        frameRate.denominator(),
        &crf.unwrap_or(23).to_string()
    );
    let mut outputVideoStream = ffOut
        .add_stream(findCodec().unwrap())
        .map_err(|e| e.to_string())?;

    let videoEncoderContext = codec::context::Context::new_with_codec(findCodec().unwrap());
    let mut videoEncoder = videoEncoderContext
        .encoder()
        .video()
        .map_err(|e| e.to_string())?;

    let alignedWidth = (width + 15) & !15;
    let alignedHeight = (height + 15) & !15;

    videoEncoder.set_width(alignedWidth);
    videoEncoder.set_height(alignedHeight);

    videoEncoder.set_format(PIXEL_FORMAT);
    videoEncoder.set_time_base(Rational::new(1, 90_000));
    videoEncoder.set_frame_rate(Some(frameRate));
    videoEncoder.set_gop((frameRate.numerator() / frameRate.denominator()) as u32);

    #[cfg(target_os = "android")]
    {
        let videoEncoder = videoEncoder.open().map_err(|e| e.to_string())?;
        outputVideoStream.set_parameters(&videoEncoder);

        unsafe {
            let streamPtr = outputVideoStream.as_mut_ptr();
            (*(*streamPtr).codecpar).width = width as i32;
            (*(*streamPtr).codecpar).height = height as i32;
        }

        return Ok(videoEncoder);
    }

    #[cfg(not(target_os = "android"))]
    {
        let mut dict = Dictionary::new();
        dict.set("crf", &crf.unwrap_or(23).to_string());
        dict.set("preset", "slow");
        let videoEncoder = videoEncoder
            .open_as_with(findCodec().unwrap(), dict)
            .map_err(|e| e.to_string())?;
        outputVideoStream.set_parameters(&videoEncoder);

        unsafe {
            let streamPtr = outputVideoStream.as_mut_ptr();
            (*(*streamPtr).codecpar).width = width as i32;
            (*(*streamPtr).codecpar).height = height as i32;
        }
        
        println!("Video encoder setup complete.");
        return Ok(videoEncoder);
    }
}

fn makeAudioEncoder(
    ffOut: &mut format::context::Output,
    audioDecoder: &decoder::Audio,
) -> Result<
    (
        Option<encoder::Audio>,
        Option<resampling::Context>,
        Option<Rational>,
    ),
    String,
> {
    let mut outputAudioStream = ffOut
        .add_stream(encoder::find(codec::Id::AAC).unwrap())
        .map_err(|e| e.to_string())?;

    let audioEncoderContext =
        codec::context::Context::new_with_codec(encoder::find(codec::Id::AAC).unwrap());
    let mut innerAudioEncoder = audioEncoderContext
        .encoder()
        .audio()
        .map_err(|e| e.to_string())?;

    innerAudioEncoder.set_rate(48000);
    innerAudioEncoder.set_channel_layout(ffmpeg::channel_layout::ChannelLayout::STEREO);
    innerAudioEncoder.set_format(format::Sample::F32(format::sample::Type::Planar));
    innerAudioEncoder.set_bit_rate(128000);
    innerAudioEncoder.set_time_base(Rational::new(1, 48000));

    let openedAudioEncoder = innerAudioEncoder
        .open_as(encoder::find(codec::Id::AAC).unwrap())
        .map_err(|e| e.to_string())?;
    outputAudioStream.set_parameters(&openedAudioEncoder);
    println!(
        "output audio stream time base: {}",
        outputAudioStream.time_base()
    );

    let resampler = resampling::context::Context::get(
        audioDecoder.format(),
        audioDecoder.channel_layout(),
        audioDecoder.rate(),
        format::Sample::F32(format::sample::Type::Planar),
        ffmpeg::channel_layout::ChannelLayout::STEREO,
        48000,
    )
    .map_err(|e| e.to_string())?;

    Ok((
        Some(openedAudioEncoder),
        Some(resampler),
        Some(outputAudioStream.time_base()),
    ))
}

pub fn processVideo<F>(input: &Video, output: String, progressCallback: F) -> Result<String, String>
where
    F: Fn(f64),
{
    // Ensure ffmpeg is initialised
    ffmpeg::init().map_err(|e| e.to_string())?;

    // Set up file access
    let mut ffIn = format::input(&input.path.to_string()).map_err(|e| e.to_string())?;
    let mut ffOut = format::output(&output).map_err(|e| e.to_string())?;

    let duration = ffIn.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64;

    // Get audio/video streams
    let inputVideo = ffIn
        .streams()
        .best(media::Type::Video)
        .ok_or("No video stream found")?;
    let videoStreamIndex = inputVideo.index();

    let inputAudio = ffIn.streams().best(media::Type::Audio);
    let audioStreamIndex = inputAudio.as_ref().map(|s| s.index());

    // Set up decoders
    let videoContextDecoder = codec::context::Context::from_parameters(inputVideo.parameters())
        .map_err(|e| e.to_string())?;
    let mut videoDecoder = videoContextDecoder
        .decoder()
        .video()
        .map_err(|e| e.to_string())?;
    let mut videoScaler = scaling::context::Context::get(
        videoDecoder.format(),
        videoDecoder.width(),
        videoDecoder.height(),
        PIXEL_FORMAT,
        input.stats.width,
        input.stats.height,
        scaling::flag::Flags::BILINEAR,
    )
    .map_err(|e| e.to_string())?;

    let mut audioDecoder = if let Some(audioStream) = inputAudio {
        let audioContextDecoder =
            codec::context::Context::from_parameters(audioStream.parameters())
                .map_err(|e| e.to_string())?;
        Some(
            audioContextDecoder
                .decoder()
                .audio()
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };

    // Set up encoders (and audio resampler)
    let mut openedVideoEncoder = makeVideoEncoder(
        &mut ffOut,
        input.stats.width,
        input.stats.height,
        Rational::new(input.stats.fps as i32, 1),
        input.crf,
    )
    .map_err(|e| e.to_string())?;

    let mut audioEncoder: Option<encoder::Audio> = None;
    let mut audioResampler: Option<resampling::Context> = None;
    let mut audioTimeBase: Option<Rational> = None;
    let aacFrameSize = 1024;
    let mut audioBuffer0: Vec<f32> = Vec::new();
    let mut audioBuffer1: Vec<f32> = Vec::new();

    if let Some(audioDecoder) = audioDecoder.as_mut() {
        (audioEncoder, audioResampler, _) =
            makeAudioEncoder(&mut ffOut, &audioDecoder).map_err(|e| e.to_string())?;
    };

    // Set up muxer
    let mut muxerOptions = Dictionary::new();
    muxerOptions.set("movflags", "+faststart");
    ffOut
        .write_header_with(muxerOptions)
        .map_err(|e| e.to_string())?;

    let videoTimeBase = ffOut.stream(0).unwrap().time_base();
    if audioEncoder.is_some() {
        audioTimeBase = Some(ffOut.stream(1).unwrap().time_base());
    }

    // Variables for frame scaling (downscaling fps specifically, upscaling is pretty pointless so we do not currently support it)
    let mut nextFramePts = 0 as i64;
    let encoderTimeBase = Rational::new(1, 90_000);
    let ptsPerFrame = encoderTimeBase.denominator() as i64 / input.stats.fps as i64;

    // Tracking for progress reports
    let mut processedFrames: u64 = 0;
    #[allow(unused)]
    let mut lastTimestamp: i64 = 0;
    let packets = ffIn.packets();

    // Main processing loop
    let mut audioPts = 0i64;
    for (stream, mut packet) in packets {
        if stream.index() == videoStreamIndex {
            // Read packets
            lastTimestamp = packet.pts().unwrap_or(0);
            videoDecoder
                .send_packet(&packet)
                .map_err(|e| e.to_string())?;

            // Decode packets
            let mut decodedFrame = ffmpeg::util::frame::video::Video::empty();
            while videoDecoder.receive_frame(&mut decodedFrame).is_ok() {
                // Scale resolution
                let mut scaledFrame = ffmpeg::util::frame::video::Video::empty();
                videoScaler
                    .run(&decodedFrame, &mut scaledFrame)
                    .map_err(|e| e.to_string())?;

                // Handle timestamping frames (and occasionally skipping in case of fps downscaling)
                let rawPts = decodedFrame.timestamp().unwrap_or(lastTimestamp);
                let pts = rawPts.rescale(stream.time_base(), openedVideoEncoder.time_base());

                if pts < nextFramePts {
                    continue;
                }

                scaledFrame.set_pts(Some(nextFramePts));
                nextFramePts += ptsPerFrame;

                // Progress report
                processedFrames += 1;
                if processedFrames % 30 == 0 {
                    let currentTime = lastTimestamp as f64
                        * (stream.time_base().numerator() as f64
                            / stream.time_base().denominator() as f64);
                    let progress = (currentTime / duration).min(1.0);

                    println!(
                        "Processed {} frames, timestamp: {:?}, progress: {}",
                        processedFrames,
                        decodedFrame.timestamp(),
                        progress
                    );
                    progressCallback(progress);
                }

                // Push processed frame to encoder
                openedVideoEncoder
                    .send_frame(&scaledFrame)
                    .map_err(|e| e.to_string())?;
            }

            // Write encoded packets
            let mut encodedPacket = ffmpeg::Packet::empty();
            while openedVideoEncoder
                .receive_packet(&mut encodedPacket)
                .is_ok()
            {
                encodedPacket.set_stream(0);
                encodedPacket.rescale_ts(openedVideoEncoder.time_base(), videoTimeBase);
                encodedPacket
                    .write_interleaved(&mut ffOut)
                    .map_err(|e| e.to_string())?;
            }
        } else if Some(stream.index()) == audioStreamIndex {
            if let Some(ref mut audioDecoder) = audioDecoder {
                if let Some(ref mut audioEncoder) = audioEncoder {
                    // Read packets
                    packet.rescale_ts(stream.time_base(), audioDecoder.time_base());
                    audioDecoder
                        .send_packet(&packet)
                        .map_err(|e| e.to_string())?;

                    // Handle decoding
                    let mut decodedFrame = ffmpeg::util::frame::audio::Audio::empty();
                    while audioDecoder.receive_frame(&mut decodedFrame).is_ok() {
                        // Resample audio
                        let mut resampledFrame = ffmpeg::util::frame::audio::Audio::new(
                            format::Sample::F32(format::sample::Type::Planar),
                            aacFrameSize,
                            ffmpeg::channel_layout::ChannelLayout::STEREO,
                        );
                        audioResampler
                            .as_mut()
                            .unwrap()
                            .run(&decodedFrame, &mut resampledFrame)
                            .map_err(|e| e.to_string())?;

                        let data0: &[f32] = resampledFrame.plane(0);
                        let data1: &[f32] = resampledFrame.plane(1);

                        // Push audio to buffer
                        for i in 0..data0.len() {
                            audioBuffer0.push(data0[i]);
                            audioBuffer1.push(data1[i]);
                        }
                    }

                    // Push buffer to upcoming frames (extra condition check is to ensure all frames are the correct size for aac encoder (1024) (last frame is allowed to be smaller))
                    while audioBuffer0.len() >= aacFrameSize && audioBuffer1.len() >= aacFrameSize {
                        let mut audioFrame = ffmpeg::util::frame::audio::Audio::new(
                            format::Sample::F32(format::sample::Type::Planar),
                            aacFrameSize,
                            ffmpeg::channel_layout::ChannelLayout::STEREO,
                        );

                        let frameData0 = audioFrame.plane_mut(0);
                        for i in 0..aacFrameSize {
                            frameData0[i] = audioBuffer0[i];
                        }

                        let frameData1 = audioFrame.plane_mut(1);
                        for i in 0..aacFrameSize {
                            frameData1[i] = audioBuffer1[i];
                        }

                        // Set timing on audio frame
                        audioFrame.set_pts(Some(audioPts));
                        audioFrame.set_rate(48000);

                        audioEncoder
                            .send_frame(&audioFrame)
                            .map_err(|e| e.to_string())?;
                        audioPts += aacFrameSize as i64;

                        audioBuffer0.drain(0..aacFrameSize);
                        audioBuffer1.drain(0..aacFrameSize);
                    }

                    // Write encoded audio to output
                    let mut encodedPacket = ffmpeg::Packet::empty();
                    while audioEncoder.receive_packet(&mut encodedPacket).is_ok() {
                        encodedPacket.set_stream(1);

                        let duration = encodedPacket.duration();
                        encodedPacket.rescale_ts(audioEncoder.time_base(), audioTimeBase.unwrap());
                        encodedPacket.set_duration(
                            duration.rescale(audioEncoder.time_base(), audioTimeBase.unwrap()),
                        );
                        encodedPacket
                            .write_interleaved(&mut ffOut)
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    // Close and flush video encoder (essentially same process as regular video encoding but without decoding or scaling more data)
    openedVideoEncoder.send_eof().map_err(|e| e.to_string())?;
    let mut encodedPacket = ffmpeg::Packet::empty();
    while openedVideoEncoder
        .receive_packet(&mut encodedPacket)
        .is_ok()
    {
        encodedPacket.set_stream(0);
        encodedPacket.rescale_ts(openedVideoEncoder.time_base(), videoTimeBase);
        encodedPacket
            .write_interleaved(&mut ffOut)
            .map_err(|e| e.to_string())?;
    }

    // Close and flush audio encoder (essentially same process as regular audio encoding but without decoding or resampling more data)
    if let Some(ref mut audioEncoder) = audioEncoder {
        if let Some(ref mut audioResampler) = audioResampler {
            let mut resampledFrame = ffmpeg::util::frame::audio::Audio::empty();

            while audioResampler.flush(&mut resampledFrame).is_ok() && resampledFrame.samples() > 0
            {
                let data0: &[f32] = resampledFrame.plane(0);
                let data1: &[f32] = resampledFrame.plane(1);

                for i in 0..data0.len() {
                    audioBuffer0.push(data0[i]);
                    audioBuffer1.push(data1[i]);
                }
            }

            while audioBuffer0.len() >= aacFrameSize {
                let mut audioFrame = ffmpeg::util::frame::audio::Audio::new(
                    format::Sample::F32(format::sample::Type::Planar),
                    aacFrameSize,
                    ffmpeg::channel_layout::ChannelLayout::STEREO,
                );

                let frameData0 = audioFrame.plane_mut(0);
                for i in 0..aacFrameSize {
                    frameData0[i] = audioBuffer0[i];
                }

                let frameData1 = audioFrame.plane_mut(1);
                for i in 0..aacFrameSize {
                    frameData1[i] = audioBuffer1[i];
                }

                audioFrame.set_pts(Some(audioPts));
                audioFrame.set_rate(48000);

                audioEncoder
                    .send_frame(&audioFrame)
                    .map_err(|e| e.to_string())?;
                audioPts += aacFrameSize as i64;

                audioBuffer0.drain(0..aacFrameSize);
                audioBuffer1.drain(0..aacFrameSize);
            }

            if !audioBuffer0.is_empty() {
                let mut audioFrame = ffmpeg::util::frame::audio::Audio::new(
                    format::Sample::F32(format::sample::Type::Planar),
                    audioBuffer0.len(),
                    ffmpeg::channel_layout::ChannelLayout::STEREO,
                );

                let frameData0 = audioFrame.plane_mut(0);
                for i in 0..audioBuffer0.len() {
                    frameData0[i] = audioBuffer0[i];
                }

                let frameData1 = audioFrame.plane_mut(1);
                for i in 0..audioBuffer1.len() {
                    frameData1[i] = audioBuffer1[i];
                }

                audioFrame.set_pts(Some(audioPts));
                audioFrame.set_rate(48000);

                audioEncoder
                    .send_frame(&audioFrame)
                    .map_err(|e| e.to_string())?;
            }

            while audioEncoder.receive_packet(&mut encodedPacket).is_ok() {
                encodedPacket.set_stream(1);

                encodedPacket.rescale_ts(audioEncoder.time_base(), audioTimeBase.unwrap());
                encodedPacket
                    .write_interleaved(&mut ffOut)
                    .map_err(|e| e.to_string())?;
            }
        }

        audioEncoder.send_eof().map_err(|e| e.to_string())?;
        while audioEncoder.receive_packet(&mut encodedPacket).is_ok() {
            encodedPacket.set_stream(1);

            encodedPacket.rescale_ts(audioEncoder.time_base(), audioTimeBase.unwrap());
            encodedPacket
                .write_interleaved(&mut ffOut)
                .map_err(|e| e.to_string())?;
        }
    }

    ffOut.write_trailer().map_err(|e| e.to_string())?;

    Ok(format!(
        "Successfully transcoded {} to {}",
        input.path.to_string(),
        output
    ))
}

pub fn getVideoStats(input: String) -> Result<VideoStats, String> {
    ffmpeg::init().map_err(|e| e.to_string())?;

    let ffIn = format::input(&input).map_err(|e| e.to_string())?;
    let videoStream = ffIn
        .streams()
        .best(media::Type::Video)
        .ok_or("No video stream found")?;
    let codecContext = ffmpeg::codec::context::Context::from_parameters(videoStream.parameters())
        .map_err(|e| e.to_string())?;
    let decoder = codecContext.decoder().video().map_err(|e| e.to_string())?;
    let frameRate = videoStream.avg_frame_rate();

    Ok(VideoStats {
        width: decoder.width(),
        height: decoder.height(),
        fps: (frameRate.numerator() as f64 / frameRate.denominator() as f64).round() as u32,
    })
}
