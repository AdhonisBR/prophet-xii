package org.prophetxii;
import android.app.Activity;
import android.os.Bundle;
import android.os.Build;
import android.content.Intent;
import android.widget.*;
import java.security.SecureRandom;
import java.net.*;
public final class MainActivity extends Activity {
 static String token(android.content.Context context){
  android.content.SharedPreferences p=context.getSharedPreferences("control",0);String t=p.getString("token",null);
  if(t==null){byte[] b=new byte[24];new SecureRandom().nextBytes(b);t=android.util.Base64.encodeToString(b,android.util.Base64.NO_WRAP|android.util.Base64.URL_SAFE);p.edit().putString("token",t).apply();}return t;
 }
 @Override public void onCreate(Bundle b){super.onCreate(b);
  if(Build.VERSION.SDK_INT>=33)requestPermissions(new String[]{"android.permission.POST_NOTIFICATIONS"},1);
  LinearLayout layout=new LinearLayout(this);layout.setOrientation(1);layout.setPadding(30,40,30,30);
  TextView text=new TextView(this);text.setTextIsSelectable(true);
  String ip="PHONE_IP";try{java.util.Enumeration<NetworkInterface> all=NetworkInterface.getNetworkInterfaces();while(all.hasMoreElements()){java.util.Enumeration<InetAddress> a=all.nextElement().getInetAddresses();while(a.hasMoreElements()){InetAddress addr=a.nextElement();if(addr instanceof Inet4Address&&!addr.isLoopbackAddress())ip=addr.getHostAddress();}}}catch(Exception ignored){}
  text.setText("PROPHET-XII · AAudio ARM64\n\nRemote control: http://"+ip+":8787\n\nAccess token (enter in browser):\n"+token(this)+"\n\nKeep the phone on a trusted network. No root required. First installation and launch require authorized Android access. Stop audio through the notification or button below.");layout.addView(text);
  Button start=new Button(this);start.setText("Start audio service");start.setOnClickListener(v->startForegroundService(new Intent(this,SynthService.class)));layout.addView(start);
  Button stop=new Button(this);stop.setText("Stop audio service");stop.setOnClickListener(v->stopService(new Intent(this,SynthService.class)));layout.addView(stop);setContentView(layout);
  startForegroundService(new Intent(this,SynthService.class));
 }
}
