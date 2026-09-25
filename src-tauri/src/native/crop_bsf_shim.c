#include <libavcodec/bsf.h>
#include <libavcodec/avcodec.h>
#include <libavutil/opt.h>

// Opaque wrapper — allocates, configures, and inits an h264_metadata BSF
// in one call, so Rust never needs to know AVBSFContext's layout.
AVBSFContext* crop_bsf_create(AVCodecParameters* par_in, int crop_right_units, int crop_bottom_units) {
    const AVBitStreamFilter *filter = av_bsf_get_by_name("h264_metadata");
    if (!filter) return NULL;

    AVBSFContext *ctx = NULL;
    if (av_bsf_alloc(filter, &ctx) < 0) return NULL;

    if (avcodec_parameters_copy(ctx->par_in, par_in) < 0) {
        av_bsf_free(&ctx);
        return NULL;
    }

    av_opt_set_int(ctx->priv_data, "crop_right", crop_right_units, 0);
    av_opt_set_int(ctx->priv_data, "crop_bottom", crop_bottom_units, 0);

    if (av_bsf_init(ctx) < 0) {
        av_bsf_free(&ctx);
        return NULL;
    }

    return ctx;
}

int crop_bsf_filter(AVBSFContext* ctx, AVPacket* pkt) {
    int ret = av_bsf_send_packet(ctx, pkt);
    if (ret < 0) return ret;
    return av_bsf_receive_packet(ctx, pkt);
}

void crop_bsf_free(AVBSFContext* ctx) {
    AVBSFContext *c = ctx;
    av_bsf_free(&c);
}