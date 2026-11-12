package org.prophetxii;
final class Native {
 static {System.loadLibrary("prophet_android");}
 static native int start();
 static native void stop();
 static native int healthy();
 static native int command(int kind,int a,int b,float value);
 static native int snapshot(float[] values);
 static native String state();
 static native String meta();
}
