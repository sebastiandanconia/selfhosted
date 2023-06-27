#!/usr/bin/python3

import os
import hashlib
import json
import time

"""
This script uses Minio's on-disk internal metadata to check the integrity of user
files.

It was made obsolete by Minio's adding support for their
Erasure Backend on single-drive installations.

TODO: Remove commented out code, add pytest.
"""

MINIO_DATA_ROOT="/mnt/DISK0/minio"
MINIO_METADATA_PREFIX=".minio.sys/buckets"

MiB=1048576
MAX_BUFFER_SIZE=8*MiB

def read_chunks(file_obj, chunk_size=4194304):
    """Generator to read a file in chunks.
    Default chunk size: 4MiB."""
    while True:
        chunk = file_obj.read(chunk_size)
        if not chunk:
            break
        yield chunk


def md5_object(path):
    h = hashlib.new('md5')
    with open(path, 'rb') as f:
        for chunk in read_chunks(f):
            h.update(chunk)
        f.close()

    return h.hexdigest()


    #The other way to get this information (More flexible/robust if using s3fs? or other clients?) is in the md5: part of the header of the API?
def md5_json_s3cmd_attr(json_path):
    f = open(json_path)
    data = json.load(f)
    attributes = dict(item.split(":") for item in data['meta']['X-Amz-Meta-S3cmd-Attrs'].split("/"))
    f.close()
    return attributes['md5']


    # s3://testing/debian-live-11.2.0-amd64-standard.iso
    # ./.minio.sys/buckets/testing/debian-live-11.2.0-amd64-standard.iso/fs.json
    # ./buckets/testing/debian-live-11.2.0-amd64-standard.iso


#A better choice for calculating etags than that below is:
# https://github.com/tlastowka/calculate_multipart_etag/blob/master/calculate_multipart_etag.py


def calculate_s3_etag(file_path, chunk_size=128 * 1024 * 1024):
    md5s = []

    with open(file_path, 'rb') as fp:
        while True:
            data = fp.read(chunk_size)
            if not data:
                break
            md5s.append(hashlib.md5(data))

    if len(md5s) < 1:
        return '"{}"'.format(hashlib.md5().hexdigest())

    if len(md5s) == 1:
        return '{}'.format(md5s[0].hexdigest())

    digests = b''.join(m.digest() for m in md5s)
    digests_md5 = hashlib.md5(digests)
    return '{}-{}'.format(digests_md5.hexdigest(), len(md5s))


def md5_meta(json_path):
    result = ""
    f = open(json_path, 'r')
    data = json.load(f)
    try:
        s3cmd_attributes = dict(item.split(":") for item in data['meta']['X-Amz-Meta-S3cmd-Attrs'].split("/"))
        result = s3cmd_attributes['md5']
    except KeyError:
        print(data['meta']['etag'])

    f.close()
    return result


def etag_meta(json_path):
    result = ""
    f = open(json_path, 'r')
    data = json.load(f)
    result = data['meta']['etag']
    f.close()
    return result


def chunk_size_meta(json_path):
    result = ""
    f = open(json_path, 'r')
    data = json.load(f)
    try:
        result = data['parts'][0]['size']
    except KeyError:
        result = None
    return result


def md5_chunk(infile, chunk_size, outfile):
    h = hashlib.new('md5')
    no_data = True
    bytes_remaining = chunk_size

    while bytes_remaining > 0:
        chunk = infile.read(min(MAX_BUFFER_SIZE, bytes_remaining))
        if chunk:
            no_data = False
            h.update(chunk)
            bytes_remaining = bytes_remaining - len(chunk)
            if outfile:
                outfile.write(chunk)
        else:
            break

    if no_data:
        return None
    else:
        return h


def calculate_multipart_etag(infile, chunk_size, outfile=None):

    """
    calculates a multipart upload etag for amazon s3

    Arguments:

    source_path -- The file to calculate the etags for
    chunk_size -- The chunk size to calculate for.

    """

    md5s = []

    while True:
        hash = md5_chunk(infile, chunk_size, outfile)
        if not hash:
            break
        md5s.append(hash)

    if len(md5s) > 1:
        digests = b"".join(m.digest() for m in md5s)
        new_md5 = hashlib.md5(digests)
        new_etag = '%s-%s' % (new_md5.hexdigest(),len(md5s))
    elif len(md5s) == 1: # file smaller than chunk size
        new_etag = '%s' % md5s[0].hexdigest()
    else: # empty file
        new_etag = ''

    return new_etag


def etag_computed(obj_path, chunk_size):
    result = ""

    f = open(obj_path, 'rb')
    if not chunk_size:
        stat = os.stat(obj_path)
        chunk_size = stat.st_size
    result = calculate_multipart_etag(f, chunk_size)
    f.close()
    return result


class S3Object:
    def __init__(self, uri, rel_meta_name, expected_etag, chunk_size, rel_obj_name):
        self.uri = uri
        self.rel_meta_name = rel_meta_name
        self.etag_expected = expected_etag
        self.rel_obj_name = rel_obj_name
        self.chunk_size = chunk_size
        self.etag_new = None
    def __str__(self):
        return '{}\nExpected etag: {}\nNew etag: {}'.format(self.uri, self.etag_expected, self.etag_new)


def discover_objects(objects):
    # TODO: Remove commented-out code and add unit tests
    for dirname, dirs, files in os.walk(MINIO_DATA_ROOT):
        dirs[:] = [d for d in dirs if not d.startswith(".")]
        for basename in files:
            if basename.startswith("."):
                continue #TODO: Test this! We want only valid objects, not metadata in hidden directories!
            full_obj_name = os.path.join(dirname, basename)
            if os.path.isfile(full_obj_name):
                rel_obj_name = os.path.relpath(full_obj_name, MINIO_DATA_ROOT)
                #print(rel_obj_name)
                rel_meta_name = os.path.relpath(full_obj_name, MINIO_DATA_ROOT)
                rel_meta_name = os.path.join(MINIO_METADATA_PREFIX, rel_meta_name)
                rel_meta_name = os.path.join(rel_meta_name, "fs.json")
                #print(rel_meta_name)
                uri_name = "s3://" + rel_obj_name
                #print(uri_name)
                etag_expected = etag_meta(os.path.join(MINIO_DATA_ROOT, rel_meta_name))
                #print("etag (json):",etag_expected)
                chunk_size = chunk_size_meta(os.path.join(MINIO_DATA_ROOT, rel_meta_name))
                #print("chunk_size:", chunk_size)
                #etag_new = etag_computed(os.path.join(MINIO_DATA_ROOT, rel_obj_name), chunk_size)
                #print("etag (computed):", etag_new)
                objects.append(S3Object(uri_name, rel_meta_name, etag_expected, chunk_size, rel_obj_name))

                #print("chunk size (json):", 


                #expected_md5 = md5_meta(os.path.join(MINIO_DATA_ROOT, rel_meta_name))
                #objects.append(S3Object(rel_meta_name, expected_md5, rel_obj_name))
                #yield (rel_obj_name, rel_meta_name)


def scrub():
    objects=[]
    discover_objects(objects)
    errors = 0
    print('Starting scrub of {}...'.format(MINIO_DATA_ROOT))
    start_time = time.time()
    """scrub repaired 0B in 00:01:11 with 0 errors on Fri Apr  1 03:46:11 2022"""
    for obj in objects:
        obj.etag_new = etag_computed(os.path.join(MINIO_DATA_ROOT, obj.rel_obj_name), obj.chunk_size)
        if obj.etag_new != obj.etag_expected:
            errors += 1
            print("********************************************************************************")
            print('* Error: {}:\n*\tcomputed etag: {}\n*\texpected etag: {}'.format(obj.uri, obj.etag_new, obj.etag_expected))
            print("********************************************************************************")

    seconds = time.time() - start_time
    minutes = seconds / 60
    hours = minutes / 60
    days = hours / 24
    seconds = seconds % 60
    print('\nScrub of {} files completed in {:.0f} days {:0>2.0f}:{:0>2.0f}:{:0>2.0f} with {} error(s).'.format(len(objects), days, hours, minutes, seconds, errors))


if __name__ == "__main__":

    scrub()
