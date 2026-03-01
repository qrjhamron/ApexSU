// IKsuInterface.aidl
package com.qrj.apexsu;

import android.content.pm.PackageInfo;
import rikka.parcelablelist.ParcelableListSlice;

interface IKsuInterface {
    ParcelableListSlice<PackageInfo> getPackages(int flags);

    int[] getUserIds();
}