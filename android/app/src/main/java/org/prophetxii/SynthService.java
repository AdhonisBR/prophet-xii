package org.prophetxii;
import android.app.*;import android.content.*;import android.os.*;import android.media.*;
import java.net.*;import java.io.*;import java.nio.charset.StandardCharsets;import java.security.MessageDigest;import java.util.*;import java.util.concurrent.*;
import org.json.*;
public final class SynthService extends Service {
 private volatile boolean running=false;private volatile String latest="{\"v\":1,\"ready\":false}";
 private ServerSocket server;private final Set<Socket> sockets=ConcurrentHashMap.newKeySet();
 private final ExecutorService workers=Executors.newFixedThreadPool(6);private final ScheduledExecutorService scheduler=Executors.newSingleThreadScheduledExecutor();
 private PowerManager.WakeLock wake;private AudioManager audioManager;private AudioFocusRequest focus;private int recoveries=0;private long lastRecovery=0;private String token;
 @Override public void onCreate(){super.onCreate();token=MainActivity.token(this);
  NotificationManager manager=getSystemService(NotificationManager.class);manager.createNotificationChannel(new NotificationChannel("audio","Synthesizer audio",NotificationManager.IMPORTANCE_LOW));
  Intent stop=new Intent(this,SynthService.class).setAction("STOP");PendingIntent action=PendingIntent.getService(this,1,stop,PendingIntent.FLAG_IMMUTABLE|PendingIntent.FLAG_UPDATE_CURRENT);
  Notification note=new Notification.Builder(this,"audio").setContentTitle("PROPHET-XII audio service").setContentText("Browser control on port 8787 · Tap Stop to end audio").setSmallIcon(android.R.drawable.ic_media_play).setOngoing(true).addAction(new Notification.Action.Builder(null,"Stop",action).build()).build();
  startForeground(12,note);
  audioManager=getSystemService(AudioManager.class);focus=new AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN).setAudioAttributes(new AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA).setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build()).setOnAudioFocusChangeListener(change->{if(change<0)stopSelf();}).build();
  if(audioManager.requestAudioFocus(focus)!=AudioManager.AUDIOFOCUS_REQUEST_GRANTED||Native.start()!=0){stopSelf();return;}
  wake=getSystemService(PowerManager.class).newWakeLock(PowerManager.PARTIAL_WAKE_LOCK,"PROPHET-XII:Audio");wake.acquire();running=true;
  try{server=new ServerSocket(8787);}catch(IOException e){stopSelf();return;}
  workers.submit(()->{while(running)try{Socket s=server.accept();s.setSoTimeout(10000);if(sockets.size()>=4){s.close();continue;}sockets.add(s);workers.submit(()->serve(s));}catch(IOException e){if(running)stopSelf();}});
  scheduler.scheduleAtFixedRate(()->{if(!running)return;String state=Native.state();if(state!=null&&state.contains("\"ready\":true"))latest=state;
   synchronized(SynthService.this){if(running&&Native.healthy()!=0&&android.os.SystemClock.elapsedRealtime()-lastRecovery>1000){lastRecovery=android.os.SystemClock.elapsedRealtime();if(recoveries++>=3){stopSelf();return;}Native.stop();if(Native.start()!=0&&recoveries>=3)stopSelf();}}
  },0,40,TimeUnit.MILLISECONDS);
 }
 @Override public int onStartCommand(Intent intent,int flags,int id){if(intent!=null&&"STOP".equals(intent.getAction()))stopSelf();return START_NOT_STICKY;}
 @Override public IBinder onBind(Intent i){return null;}
 @Override public synchronized void onDestroy(){running=false;try{if(server!=null)server.close();for(Socket s:sockets)s.close();}catch(IOException ignored){}scheduler.shutdownNow();workers.shutdownNow();Native.stop();if(wake!=null&&wake.isHeld())wake.release();if(audioManager!=null&&focus!=null)audioManager.abandonAudioFocusRequest(focus);stopForeground(STOP_FOREGROUND_REMOVE);super.onDestroy();}
 private static byte[] readBytes(InputStream in,int length)throws IOException{byte[] out=new byte[length];int pos=0;while(pos<length){int n=in.read(out,pos,length-pos);if(n<0)throw new EOFException();pos+=n;}return out;}
 private static byte[] readAsset(InputStream in)throws IOException{ByteArrayOutputStream out=new ByteArrayOutputStream();byte[] chunk=new byte[8192];int n;while((n=in.read(chunk))!=-1){out.write(chunk,0,n);if(out.size()>2*1024*1024)throw new IOException("asset size");}return out.toByteArray();}
 private static String line(InputStream in)throws IOException{ByteArrayOutputStream b=new ByteArrayOutputStream();for(int i=0;i<8192;i++){int c=in.read();if(c==-1)throw new EOFException();if(c=='\n')return b.toString("UTF-8").trim();b.write(c);}throw new IOException("header too long");}
 private static void response(OutputStream out,int code,String type,byte[] data)throws IOException{out.write(("HTTP/1.1 "+code+" OK\r\nContent-Type: "+type+"\r\nContent-Length: "+data.length+"\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n").getBytes(StandardCharsets.US_ASCII));out.write(data);out.flush();}
 private void serve(Socket socket){try(Socket s=socket){InputStream in=s.getInputStream();OutputStream out=s.getOutputStream();String[] request=line(in).split(" ");if(request.length<2)return;URI uri=URI.create(request[1]);String path=uri.getPath();Map<String,String> headers=new HashMap<>();int total=0;for(;;){String h=line(in);total+=h.length();if(total>16384)throw new IOException("headers too long");if(h.isEmpty())break;int c=h.indexOf(':');if(c>0)headers.put(h.substring(0,c).toLowerCase(Locale.ROOT),h.substring(c+1).trim());}
  boolean auth=("Bearer "+token).equals(headers.get("authorization"));if(uri.getRawQuery()!=null)for(String q:uri.getRawQuery().split("&")){String[] v=q.split("=",2);if(v.length==2&&v[0].equals("token")&&URLDecoder.decode(v[1],"UTF-8").equals(token))auth=true;}
  if(path.startsWith("/api/")&&!path.equals("/api/v1/meta")&&!auth){response(out,401,"application/json","{\"error\":\"unauthorized\"}".getBytes());return;}
  if(path.equals("/api/v1/ws")){String key=headers.get("sec-websocket-key");if(key==null)return;String accept=android.util.Base64.encodeToString(MessageDigest.getInstance("SHA-1").digest((key+"258EAFA5-E914-47DA-95CA-C5AB0DC85B11").getBytes(StandardCharsets.US_ASCII)),android.util.Base64.NO_WRAP);out.write(("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: "+accept+"\r\n\r\n").getBytes(StandardCharsets.US_ASCII));out.flush();
   try{while(running){if(in.available()>0){int first=in.read(),len=in.read();if(first<0||len<0||(first&15)==8)break;int n=len&127;if(n==126)n=(in.read()<<8)|in.read();if(n==127)break;int skip=n+((len&128)!=0?4:0);while(skip-->0)if(in.read()<0)throw new EOFException();}byte[] data=latest.getBytes(StandardCharsets.UTF_8);out.write(0x81);if(data.length<126)out.write(data.length);else{out.write(126);out.write(data.length>>8);out.write(data.length&255);}out.write(data);out.flush();Thread.sleep(40);}}finally{Native.command(3,0,0,0);}return;
  }
  if(path.equals("/api/v1/state")){response(out,200,"application/json",latest.getBytes(StandardCharsets.UTF_8));return;}
  if(path.equals("/api/v1/meta")){response(out,200,"application/json",Native.meta().getBytes(StandardCharsets.UTF_8));return;}
  if(path.equals("/api/v1/command")&&request[0].equals("POST")){int length=Integer.parseInt(headers.getOrDefault("content-length","0"));if(length<1||length>32768)throw new IOException("body size");byte[] data=readBytes(in,length);if(data.length!=length)throw new EOFException();JSONObject command=new JSONObject(new String(data,StandardCharsets.UTF_8));String kind=command.getString("type");int result=-2;
   switch(kind){case "param":result=Native.command(0,command.getInt("id"),0,(float)command.getDouble("value"));break;case "midi":result=Native.command(1,command.getInt("status"),command.getInt("data1"),command.optInt("data2",0));break;case "preset":result=Native.command(2,command.getInt("index"),0,0);break;case "panic":result=Native.command(3,0,0,0);break;case "stop":result=0;new Handler(Looper.getMainLooper()).postDelayed(()->stopSelf(),150);break;case "trimpot":result=Native.command(4,command.getInt("voice"),command.getInt("field"),(float)command.getDouble("value"));break;case "learn":result=Native.command(5,command.getInt("id"),0,0);break;case "snapshot":JSONArray values=command.getJSONArray("values");float[] a=new float[values.length()];for(int i=0;i<a.length;i++)a[i]=(float)values.getDouble(i);result=Native.snapshot(a);break;}
   response(out,result==0?202:400,"application/json",("{\"result\":"+result+"}").getBytes());return;
  }
  if(path.startsWith("/api/")){response(out,404,"application/json","{\"error\":\"unsupported Android endpoint\"}".getBytes());return;}
  if(path.contains(".."))return;String asset=path.equals("/")?"index.html":path.substring(1);try(InputStream file=getAssets().open(asset)){String type=asset.endsWith(".js")?"text/javascript":asset.endsWith(".css")?"text/css":asset.endsWith(".html")?"text/html":"application/octet-stream";response(out,200,type,readAsset(file));}
 }catch(Exception ignored){}finally{sockets.remove(socket);}}
}
