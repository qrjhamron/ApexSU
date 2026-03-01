# Keep all JNI-bound classes and their members.
# Rust JNI bridge accesses Profile fields by name via env.set_field().
-keep class com.qrj.apexsu.Natives { *; }
-keep class com.qrj.apexsu.Natives$Profile { *; }
-keep class com.qrj.apexsu.Natives$Profile$* { *; }

# Keep AIDL-generated interfaces used by RootService IPC
-keep class com.qrj.apexsu.IKsuInterface { *; }
-keep class com.qrj.apexsu.IKsuInterface$* { *; }

# Keep Parcelable CREATOR fields
-keepclassmembers class * implements android.os.Parcelable {
    public static final ** CREATOR;
}

# Keep kotlinx.serialization generated code
-keepclassmembers @kotlinx.serialization.Serializable class ** {
    *** Companion;
    *** serializer(...);
    kotlinx.serialization.KSerializer serializer(...);
}
-keep class **$$serializer { *; }