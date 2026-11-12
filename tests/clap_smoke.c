// SPDX-License-Identifier: GPL-3.0-only
#include <clap/clap.h>
#include <assert.h>
#include <dlfcn.h>
#include <math.h>
#include <stdio.h>
#include <string.h>
static const void *extension(const clap_host_t *h,const char *id){return NULL;}
static void noop(const clap_host_t *h){}
static clap_event_note_t note;
static uint32_t count(const clap_input_events_t *l){return l->ctx?1:0;}
static const clap_event_header_t *get(const clap_input_events_t *l,uint32_t i){return &note.header;}
static bool push(const clap_output_events_t *l,const clap_event_header_t *e){return true;}
struct Memory{unsigned char bytes[65536];size_t len,pos;};
static int64_t write_state(const clap_ostream_t *s,const void *data,uint64_t n){struct Memory *m=s->ctx;if(m->len+n>sizeof m->bytes)return -1;memcpy(m->bytes+m->len,data,n);m->len+=n;return n;}
static int64_t read_state(const clap_istream_t *s,void *data,uint64_t n){struct Memory *m=s->ctx;if(n>m->len-m->pos)n=m->len-m->pos;memcpy(data,m->bytes+m->pos,n);m->pos+=n;return n;}
int main(int argc,char **argv){assert(argc==2);void *lib=dlopen(argv[1],RTLD_NOW);if(!lib){fprintf(stderr,"%s\n",dlerror());return 1;}
 const clap_plugin_entry_t *entry=dlsym(lib,"clap_entry");assert(entry&&entry->init(argv[1]));const clap_plugin_factory_t *factory=entry->get_factory(CLAP_PLUGIN_FACTORY_ID);assert(factory&&factory->get_plugin_count(factory)==1);const clap_plugin_descriptor_t *desc=factory->get_plugin_descriptor(factory,0);assert(desc);clap_host_t host={.clap_version=CLAP_VERSION_INIT,.name="PROPHET-XII ABI test host",.vendor="Local test",.url="https://localhost",.version="1",.get_extension=extension,.request_restart=noop,.request_process=noop,.request_callback=noop};
 const clap_plugin_t *plugin=factory->create_plugin(factory,&host,desc->id);assert(plugin&&plugin->init(plugin));const clap_plugin_params_t *params=plugin->get_extension(plugin,CLAP_EXT_PARAMS);assert(params);uint32_t total=params->count(plugin);assert(total>=79);const clap_plugin_state_t *state=plugin->get_extension(plugin,CLAP_EXT_STATE);assert(state);struct Memory mem={0};clap_ostream_t out={.ctx=&mem,.write=write_state};assert(state->save(plugin,&out)&&mem.len>0);clap_istream_t in={.ctx=&mem,.read=read_state};assert(state->load(plugin,&in));assert(plugin->activate(plugin,48000,1,256));assert(plugin->start_processing(plugin));
 float l[256]={0},r[256]={0};float *channels[]={l,r};clap_audio_buffer_t audio={.data32=channels,.channel_count=2};clap_input_events_t events={.ctx=(void*)1,.size=count,.get=get};clap_output_events_t output={.try_push=push};clap_process_t process={.steady_time=-1,.frames_count=256,.audio_outputs=&audio,.audio_outputs_count=1,.in_events=&events,.out_events=&output};
 note=(clap_event_note_t){.header={.size=sizeof(note),.time=17,.space_id=CLAP_CORE_EVENT_SPACE_ID,.type=CLAP_EVENT_NOTE_ON},.note_id=-1,.port_index=0,.channel=0,.key=60,.velocity=.8};double energy=0;
 for(int block=0;block<100;block++){assert(plugin->process(plugin,&process)!=CLAP_PROCESS_ERROR);for(int i=0;i<256;i++){assert(isfinite(l[i])&&isfinite(r[i]));if(block==0&&i<17)assert(l[i]==0&&r[i]==0);energy+=l[i]*l[i]+r[i]*r[i];}events.ctx=NULL;}
 assert(energy>.1);const clap_plugin_latency_t *latency=plugin->get_extension(plugin,CLAP_EXT_LATENCY);assert(latency);printf("PASS: CLAP descriptor, %u parameters, %zu-byte state save/restore, MIDI at offset 17, stereo energy %.6f, latency %u samples\n",total,mem.len,energy,latency->get(plugin));plugin->stop_processing(plugin);plugin->deactivate(plugin);plugin->destroy(plugin);entry->deinit();dlclose(lib);return 0;
}
