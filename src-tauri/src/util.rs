use ffmpeg::{
    codec, decoder, encoder, format, media, software::resampling, Dictionary, Rational, Rescale,
};
use ffmpeg_next as ffmpeg;

fn makeVideoEncoder(
    ffOut: &mut format::context::Output,
    width: u32,
    height: u32,
    frameRate: Rational,
) -> Result<(encoder::Video, Rational), String> {
    println!(
        "Setting up video encoder with width: {}, height: {}, frame rate: {}/{}",
        width,
        height,
        frameRate.numerator(),
        frameRate.denominator()
    );
    let mut outputVideoStream = ffOut
        .add_stream(encoder::find(codec::Id::H264).unwrap())
        .map_err(|e| e.to_string())?;
    let videoTimeBase = outputVideoStream.time_base();

    let videoEncoderContext =
        codec::context::Context::new_with_codec(encoder::find(codec::Id::H264).unwrap());
    let mut videoEncoder = videoEncoderContext
        .encoder()
        .video()
        .map_err(|e| e.to_string())?;

    println!("Setting video encoder parameters...");

    videoEncoder.set_width(width);
    videoEncoder.set_height(height);
    videoEncoder.set_format(format::Pixel::YUV420P);
    // videoEncoder.set_time_base(Rational::new(frameRate.denominator(), frameRate.numerator()));
    // videoEncoder.set_time_base(frameRate.invert());
    videoEncoder.set_time_base(Rational::new(1, 60_000));
    videoEncoder.set_frame_rate(Some(frameRate));
    videoEncoder.set_gop((frameRate.numerator() / frameRate.denominator()) as u32);

    let mut videoOptions = Dictionary::new();
    videoOptions.set("crf", "23");
    videoOptions.set("preset", "medium");

    println!("Opening video encoder...");

    let videoEncoder = videoEncoder
        .open_with(videoOptions)
        .map_err(|e| e.to_string())?;
    outputVideoStream.set_parameters(&videoEncoder);

    println!(
        "Video encoder setup complete. Stream time base: {}",
        outputVideoStream.time_base()
    );

    Ok((videoEncoder, videoTimeBase))
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

pub fn processVideo<F>(input: String, output: String, progressCallback: F) -> Result<String, String>
where
    F: Fn(f64),
{
    ffmpeg::init().map_err(|e| e.to_string())?;

    let mut ffIn = format::input(&input).map_err(|e| e.to_string())?;
    let mut ffOut = format::output(&output).map_err(|e| e.to_string())?;

    let duration = ffIn.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64;

    let inputVideo = ffIn
        .streams()
        .best(media::Type::Video)
        .ok_or("No video stream found")?;
    let videoStreamIndex = inputVideo.index();

    let inputAudio = ffIn.streams().best(media::Type::Audio);
    let audioStreamIndex = inputAudio.as_ref().map(|s| s.index());

    let videoContextDecoder = codec::context::Context::from_parameters(inputVideo.parameters())
        .map_err(|e| e.to_string())?;
    let mut videoDecoder = videoContextDecoder
        .decoder()
        .video()
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

    let (mut openedVideoEncoder, _) = makeVideoEncoder(
        &mut ffOut,
        videoDecoder.width(),
        videoDecoder.height(),
        inputVideo.avg_frame_rate(),
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

    let mut muxerOptions = Dictionary::new();
    muxerOptions.set("movflags", "+faststart");
    ffOut
        .write_header_with(muxerOptions)
        .map_err(|e| e.to_string())?;

    let videoTimeBase = ffOut.stream(0).unwrap().time_base();
    if audioEncoder.is_some() {
        audioTimeBase = Some(ffOut.stream(1).unwrap().time_base());
    }

    let mut processedFrames: u64 = 0;
    #[allow(unused)]
    let mut lastTimestamp: i64 = 0;
    let packets = ffIn.packets();

    let mut audioPts = 0i64;
    for (stream, mut packet) in packets {
        if stream.index() == videoStreamIndex {
            lastTimestamp = packet.pts().unwrap_or(0);
            videoDecoder
                .send_packet(&packet)
                .map_err(|e| e.to_string())?;

            let mut decodedFrame = ffmpeg::util::frame::video::Video::empty();
            while videoDecoder.receive_frame(&mut decodedFrame).is_ok() {
                let rawPts = decodedFrame.timestamp().unwrap_or(lastTimestamp);
                let pts = rawPts.rescale(stream.time_base(), openedVideoEncoder.time_base());
                decodedFrame.set_pts(Some(pts));

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

                openedVideoEncoder
                    .send_frame(&decodedFrame)
                    .map_err(|e| e.to_string())?;
            }

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
                    packet.rescale_ts(stream.time_base(), audioDecoder.time_base());
                    audioDecoder
                        .send_packet(&packet)
                        .map_err(|e| e.to_string())?;

                    let mut decodedFrame = ffmpeg::util::frame::audio::Audio::empty();
                    while audioDecoder.receive_frame(&mut decodedFrame).is_ok() {
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

                        for i in 0..data0.len() {
                            audioBuffer0.push(data0[i]);
                            audioBuffer1.push(data1[i]);
                        }
                    }

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

                        audioFrame.set_pts(Some(audioPts));
                        audioFrame.set_rate(48000);

                        audioEncoder
                            .send_frame(&audioFrame)
                            .map_err(|e| e.to_string())?;
                        audioPts += aacFrameSize as i64;

                        audioBuffer0.drain(0..aacFrameSize);
                        audioBuffer1.drain(0..aacFrameSize);
                    }

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

    Ok(format!("Successfully transcoded {} to {}", input, output))
}
